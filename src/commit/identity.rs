//! commit author/committer 身份保留。
//!
//! `git-reword` / `git-retime` 只改 message 或时间戳，**不**读取 `user.name` / `user.email`，
//! 也不增删贡献者。

use gix::actor::Signature;

/// 完整复制签名（name、email、time 均不变）。
pub fn copy_signature(signature: Signature) -> Signature {
    Signature { name: signature.name, email: signature.email, time: signature.time }
}

/// 仅替换 `time.seconds`；name、email、offset、sign 与原文一致。
pub fn retime_signature(original: Signature, seconds: i64) -> Signature {
    Signature {
        name: original.name,
        email: original.email,
        time: gix::date::Time { seconds, offset: original.time.offset, sign: original.time.sign },
    }
}

/// 比较两个签名的贡献者身份（忽略时间戳）。
pub fn same_contributor(left: &Signature, right: &Signature) -> bool {
    left.name == right.name && left.email == right.email
}

#[cfg(test)]
mod tests {
    use super::{copy_signature, retime_signature, same_contributor};

    fn sample_signature() -> gix::actor::Signature {
        gix::actor::Signature {
            name: "Alice".into(),
            email: "alice@example.com".into(),
            time: gix::date::Time { seconds: 1_000, offset: 28_800, sign: gix::date::time::Sign::Plus },
        }
    }

    #[test]
    fn copy_signature_is_unchanged() {
        let original = sample_signature();
        let copied = copy_signature(original.clone());
        assert!(same_contributor(&original, &copied));
        assert_eq!(copied.time.seconds, original.time.seconds);
        assert_eq!(copied.time.offset, original.time.offset);
        assert_eq!(copied.time.sign, original.time.sign);
    }

    #[test]
    fn retime_signature_preserves_identity() {
        let original = sample_signature();
        let updated = retime_signature(original.clone(), 9_999);
        assert!(same_contributor(&original, &updated));
        assert_eq!(updated.time.seconds, 9_999);
        assert_eq!(updated.time.offset, original.time.offset);
        assert_eq!(updated.time.sign, original.time.sign);
    }
}
