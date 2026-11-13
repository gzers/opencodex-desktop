//! Exact Windows PATH policy; original type and UTF-16 bytes never normalized.
use super::fail;
use crate::errors::AppResult;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PathValue {
    pub value_type: u32,
    pub bytes: Vec<u8>,
}
impl PathValue {
    pub(super) fn text(&self) -> AppResult<String> {
        if ![1, 2].contains(&self.value_type)
            || !(2..=256 * 1024).contains(&self.bytes.len())
            || !self.bytes.len().is_multiple_of(2)
        {
            return Err(fail("用户 PATH 类型或长度不明确"));
        }
        let units: Vec<u16> = self
            .bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| u16::from_le_bytes(*v))
            .collect();
        if units.last() != Some(&0) || units[..units.len() - 1].contains(&0) {
            return Err(fail("用户 PATH 终止格式不明确"));
        }
        String::from_utf16(&units[..units.len() - 1]).map_err(|_| fail("用户 PATH 编码不明确"))
    }
    pub(super) fn encode(kind: u32, text: &str) -> AppResult<Self> {
        let value = Self {
            value_type: kind,
            bytes: text
                .encode_utf16()
                .chain(Some(0))
                .flat_map(u16::to_le_bytes)
                .collect(),
        };
        value.text()?;
        Ok(value)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PathEdit {
    pub before: Option<PathValue>,
    pub after: Option<PathValue>,
    pub inserted: bool,
}
pub(super) fn unquote(text: &str) -> AppResult<&str> {
    let plain = if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        &text[1..text.len() - 1]
    } else {
        text
    };
    if plain.contains('"') {
        return Err(fail("PATH 引号位置不明确"));
    }
    Ok(plain)
}
pub(super) fn parts(text: &str) -> AppResult<Vec<&str>> {
    if text.chars().any(char::is_control) {
        return Err(fail("PATH 含控制字符"));
    }
    let (mut result, mut start, mut quoted) = (vec![], 0, false);
    for (i, ch) in text.char_indices() {
        if ch == '"' {
            quoted = !quoted;
        }
        if ch == ';' && !quoted {
            result.push(&text[start..i]);
            start = i + 1;
        }
    }
    if quoted {
        return Err(fail("PATH 引号未闭合"));
    }
    result.push(&text[start..]);
    for p in &result {
        unquote(p)?;
    }
    Ok(result)
}
fn equivalent(a: &str, b: &str) -> AppResult<bool> {
    let fold = |s: &str| {
        let mut s = s.replace('/', "\\").to_lowercase();
        while s.ends_with('\\') && s.len() > 3 {
            s.pop();
        }
        s
    };
    Ok(fold(unquote(a)?) == fold(unquote(b)?))
}
fn segment(directory: &str, kind: u32) -> AppResult<String> {
    if directory.is_empty()
        || directory.chars().any(char::is_control)
        || directory.contains('"')
        || kind == 2 && directory.contains('%')
    {
        return Err(fail("目录无法按原 PATH 类型无损表示"));
    }
    Ok(if directory.contains(';') {
        format!("\"{directory}\"")
    } else {
        directory.into()
    })
}
pub(super) fn enable(before: &Option<PathValue>, directory: &str) -> AppResult<PathEdit> {
    let kind = before.as_ref().map_or(1, |v| v.value_type);
    let own = segment(directory, kind)?;
    let text = before
        .as_ref()
        .map(PathValue::text)
        .transpose()?
        .unwrap_or_default();
    let found = parts(&text)?
        .iter()
        .map(|p| equivalent(p, &own))
        .collect::<AppResult<Vec<_>>>()?
        .into_iter()
        .filter(|v| *v)
        .count();
    if found > 1 {
        return Err(fail("PATH 含重复的本应用目录"));
    }
    if found == 1 {
        return Ok(PathEdit {
            before: before.clone(),
            after: before.clone(),
            inserted: false,
        });
    }
    let separator = if text.is_empty() || text.ends_with(';') {
        ""
    } else {
        ";"
    };
    Ok(PathEdit {
        before: before.clone(),
        after: Some(PathValue::encode(kind, &format!("{text}{separator}{own}"))?),
        inserted: true,
    })
}
pub(super) fn validate(edit: &PathEdit, directory: &str) -> AppResult<()> {
    if enable(&edit.before, directory)? != *edit {
        return Err(fail("PATH 登记不是原预览的精确增量"));
    }
    Ok(())
}
pub(super) fn disable(
    edit: &PathEdit,
    current: &Option<PathValue>,
    directory: &str,
) -> AppResult<Option<PathValue>> {
    validate(edit, directory)?;
    if !edit.inserted {
        if let Some(v) = current {
            v.text()?;
        }
        return Ok(current.clone());
    }
    if *current == edit.after {
        return Ok(edit.before.clone());
    }
    let value = current
        .as_ref()
        .ok_or_else(|| fail("原 PATH 条目已被删除，未覆盖"))?;
    if edit.after.as_ref().map(|v| v.value_type) != Some(value.value_type) {
        return Err(fail("PATH 类型已变化，未覆盖"));
    }
    let text = value.text()?;
    let own = segment(directory, value.value_type)?;
    let mut pieces = parts(&text)?;
    let found = pieces
        .iter()
        .enumerate()
        .map(|(i, p)| equivalent(p, &own).map(|same| (i, same)))
        .collect::<AppResult<Vec<_>>>()?
        .into_iter()
        .filter_map(|(i, same)| same.then_some(i))
        .collect::<Vec<_>>();
    if found.len() != 1 || pieces[found[0]] != own {
        return Err(fail("本应用 PATH 条目已修改或重复，未删除"));
    }
    pieces.remove(found[0]);
    Ok(Some(PathValue::encode(
        value.value_type,
        &pieces.join(";"),
    )?))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_types_absence_empty_and_separators_restore_exactly() {
        for before in [
            None,
            Some(PathValue::encode(1, "").unwrap()),
            Some(PathValue::encode(2, "%USERPROFILE%\\tools;").unwrap()),
            Some(PathValue::encode(1, "C:\\other;;").unwrap()),
        ] {
            let e = enable(&before, "C:\\our 雪\\bin").unwrap();
            assert!(e.inserted);
            assert_eq!(disable(&e, &e.after, "C:\\our 雪\\bin").unwrap(), before);
        }
    }
    #[test]
    fn equivalent_preexisting_directory_is_not_owned_or_rewritten() {
        let before = Some(PathValue::encode(2, "C:/Tools/Bin/;X:\\Other").unwrap());
        let e = enable(&before, "c:\\tools\\bin").unwrap();
        assert!(!e.inserted);
        assert_eq!(e.after, before);
        let later = Some(PathValue::encode(2, "outside").unwrap());
        assert_eq!(disable(&e, &later, "c:\\tools\\bin").unwrap(), later);
    }
    #[test]
    fn unrelated_later_edits_survive_but_changed_owned_entry_refuses() {
        let e = enable(&Some(PathValue::encode(1, "C:\\old").unwrap()), "C:\\ours").unwrap();
        let current = Some(PathValue::encode(1, "X:\\new;C:\\ours;Y:\\last;").unwrap());
        assert_eq!(
            disable(&e, &current, "C:\\ours")
                .unwrap()
                .unwrap()
                .text()
                .unwrap(),
            "X:\\new;Y:\\last;"
        );
        for s in ["C:\\OURS", "C:\\ours;C:\\ours", "elsewhere"] {
            assert!(disable(&e, &Some(PathValue::encode(1, s).unwrap()), "C:\\ours").is_err());
        }
        assert!(disable(&e, &None, "C:\\ours").is_err());
    }
    #[test]
    fn quotes_semicolons_malformed_utf16_and_forged_edits_refuse() {
        let e = enable(&None, "C:\\a;b").unwrap();
        assert_eq!(e.after.as_ref().unwrap().text().unwrap(), "\"C:\\a;b\"");
        assert_eq!(disable(&e, &e.after, "C:\\a;b").unwrap(), None);
        for s in ["\"open", "bad\"quote\"", "line\nnext"] {
            assert!(enable(&Some(PathValue::encode(1, s).unwrap()), "C:\\ours").is_err());
        }
        assert!(enable(
            &Some(PathValue::encode(2, "").unwrap()),
            "C:\\%future%\\bin"
        )
        .is_err());
        for value in [
            PathValue {
                value_type: 3,
                bytes: vec![0, 0],
            },
            PathValue {
                value_type: 1,
                bytes: vec![0],
            },
            PathValue {
                value_type: 1,
                bytes: vec![0, 0, 0, 0],
            },
            PathValue {
                value_type: 1,
                bytes: vec![0, 0xd8, 0, 0],
            },
        ] {
            assert!(value.text().is_err());
        }
        let mut fake = e;
        fake.before = Some(PathValue::encode(1, "arbitrary").unwrap());
        assert!(validate(&fake, "C:\\a;b").is_err());
    }
}
