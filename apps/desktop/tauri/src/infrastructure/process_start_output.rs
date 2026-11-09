//! Fixed-field observations of child output; never exports upstream text.

#[derive(Default)]
pub(super) struct OutputStream {
    pub bytes: u64,
    pub eof: bool,
    pub read_failed: bool,
    pub admission_failure: Option<String>,
    tail: zeroize::Zeroizing<Vec<u8>>,
    kind: Option<&'static str>,
    hint: Option<&'static str>,
    site: Option<(&'static str, u32, u32)>,
}

impl OutputStream {
    pub fn push(&mut self, bytes: &[u8]) {
        self.bytes = self.bytes.saturating_add(bytes.len() as u64);
        self.tail.extend_from_slice(bytes);
        if self.tail.len() > 8192 {
            let discard = self.tail.len() - 8192;
            self.tail.drain(..discard);
        }
        let text =
            zeroize::Zeroizing::new(String::from_utf8_lossy(&self.tail).to_ascii_lowercase());
        for raw_line in text.split_inclusive('\n') {
            let complete_line = raw_line.ends_with('\n');
            let line = raw_line.trim_end_matches(['\r', '\n']);
            let line = line.trim_start();
            if complete_line {
                if let Some(value) = line.strip_prefix("ocxd-lifetime-admission-failed stage=") {
                    if super::process_runner::valid_admission_diagnostic(value) {
                        self.admission_failure = Some(value.to_owned());
                    }
                }
            }
            if self.kind.is_none() {
                self.kind = [
                    ("referenceerror:", "reference"),
                    ("typeerror:", "type"),
                    ("syntaxerror:", "syntax"),
                    ("rangeerror:", "range"),
                    ("error:", "error"),
                ]
                .into_iter()
                .find_map(|(prefix, kind)| line.starts_with(prefix).then_some(kind));
            }
            if self.hint.is_none() {
                self.hint = [
                    ("cannot access", "initialization"),
                    ("is not defined", "undefined-name"),
                    ("is not a function", "not-callable"),
                    ("cannot find package", "package-resolution"),
                    ("cannot find module", "module-resolution"),
                    ("cannot resolve", "module-resolution"),
                    ("module_not_found", "module-resolution"),
                    ("failed to launch bun runtime", "bun-launch"),
                    (
                        "windows runtime lifetime authority cannot be proved",
                        "lifetime-authority",
                    ),
                    ("cannot open shared object", "native-library"),
                    ("failed to open library", "native-library"),
                    ("symbol not found", "native-symbol"),
                    ("eacces", "permission"),
                    ("access is denied", "permission"),
                    ("enoent", "missing-file"),
                ]
                .into_iter()
                .find_map(|(pattern, hint)| line.contains(pattern).then_some(hint));
            }
            if self.site.is_none() && (complete_line || line.ends_with(')')) {
                // Only reviewed runtime source suffixes and numeric positions.
                // No arbitrary path, symbol, message, source excerpt or config.
                for name in [
                    "bin/ocx.mjs",
                    "src/lib/windows-runtime-start-boot.ts",
                    "src/lib/windows-runtime-lifetime.ts",
                    "src/lib/windows-runtime-private-record.ts",
                    "src/cli/index.ts",
                    "src/config/atomic-write.ts",
                    "src/config/process-state.ts",
                ] {
                    let normalized = line.replace('\\', "/");
                    if let Some((_, location)) = normalized.split_once(&format!("/{name}:")) {
                        let mut parts = location.split(':');
                        let row = number(parts.next().unwrap_or(""));
                        let column = number(parts.next().unwrap_or(""));
                        if let (Some(row), Some(column)) = (row, column) {
                            self.site = Some((name, row, column));
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn summary(&self, name: &str) -> String {
        let site = self.site.map_or_else(
            || "unobserved".to_string(),
            |(file, row, column)| format!("{file}:{row}:{column}"),
        );
        format!(
            "{name}[bytes={},eof={},read_failed={},kind={},hint={},site={site}]",
            self.bytes,
            self.eof,
            self.read_failed,
            self.kind.unwrap_or("unobserved"),
            self.hint.unwrap_or("unobserved")
        )
    }
}

fn number(text: &str) -> Option<u32> {
    let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() || digits.len() > 10 {
        return None;
    }
    digits.parse().ok().filter(|value| *value > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_error_and_windows_stack_export_only_fixed_fields() {
        let mut output = OutputStream::default();
        output.push(b"ReferenceEr");
        output.push(b"ror: Cannot access 'private-key' before initialization\n    at C:\\Users\\secret\\src\\lib\\windows-runtime-lifetime.ts:121:9)\n");
        let summary = output.summary("stderr");
        assert!(summary.contains("kind=reference,hint=initialization"));
        assert!(summary.contains("site=src/lib/windows-runtime-lifetime.ts:121:9"));
        assert!(!summary.contains("secret") && !summary.contains("private-key"));
    }

    #[test]
    fn unknown_messages_paths_and_control_characters_never_escape() {
        let mut output = OutputStream::default();
        output.push(b"Error: token=PRIVATE Authorization: Bearer PRIVATE\n/unknown/private.ts:12:4\n\x1b[31mPRIVATE");
        let summary = output.summary("stderr");
        assert!(summary.contains("kind=error,hint=unobserved,site=unobserved"));
        assert!(!summary.contains("PRIVATE") && !summary.contains("private.ts"));
        assert!(!summary.contains('\u{1b}'));
    }

    #[test]
    fn split_frame_numbers_are_not_committed_before_the_line_completes() {
        let mut output = OutputStream::default();
        output.push(b"    at C:/PRIVATE/src/cli/index.ts:12:9");
        assert!(output.summary("stderr").contains("site=unobserved"));
        output.push(b"0)\n");
        assert!(output
            .summary("stderr")
            .contains("site=src/cli/index.ts:12:90"));
    }

    #[test]
    fn empty_incomplete_and_failed_reads_are_distinct_and_retention_is_bounded() {
        let mut output = OutputStream::default();
        assert!(output
            .summary("stdout")
            .contains("bytes=0,eof=false,read_failed=false"));
        output.read_failed = true;
        assert!(output
            .summary("stdout")
            .contains("eof=false,read_failed=true"));
        for _ in 0..16 {
            output.push(&[b'x'; 2048]);
        }
        assert_eq!(output.bytes, 32768);
        assert_eq!(output.tail.len(), 8192);
        output.eof = true;
        assert!(output.summary("stdout").contains("eof=true"));
    }
}
