//! Independent r7 private protocol. Decoded requests/locator/clock are DATA,
//! never an original Owner, native Job, recovery proof or action admission.
//! Original authenticated same-image/ordinary/no-Job transport must bind them.
mod wire;
