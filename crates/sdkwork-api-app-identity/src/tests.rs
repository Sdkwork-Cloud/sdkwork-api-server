use super::validate_password_strength;

#[test]
fn validate_password_strength_rejects_weak_password_shapes() {
    let cases = [
        ("Short1!", "password must be at least 12 characters"),
        ("password1234!", "password must include an uppercase letter"),
        ("PASSWORD1234!", "password must include a lowercase letter"),
        ("PasswordOnly!!", "password must include a number"),
        ("Password1234", "password must include a special character"),
        ("Password 123!", "password must not contain whitespace"),
    ];

    for (password, expected) in cases {
        let error = validate_password_strength(password).unwrap_err();
        assert_eq!(error, expected, "password `{password}`");
    }
}

#[test]
fn validate_password_strength_accepts_strong_passwords() {
    assert!(validate_password_strength("Password1234!").is_ok());
    assert!(validate_password_strength("ChangeMe123!").is_ok());
}

