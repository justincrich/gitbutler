use but_authz::{Authority, AuthoritySet, ParseAuthorityError};

#[test]
fn authority_parse() {
    assert_eq!(
        Authority::parse("contents:write"),
        Ok(Authority::ContentsWrite),
        "contents:write must resolve to its exact functional authority"
    );
    assert_eq!(
        Authority::parse("administration:write"),
        Ok(Authority::AdministrationWrite),
        "administration:write must resolve to the admin-write authority"
    );
    assert_eq!(
        Authority::parse("merge"),
        Ok(Authority::Merge),
        "merge must be present in the functional catalog"
    );

    let error = Authority::parse("contents:bogus")
        .err()
        .unwrap_or_else(|| ParseAuthorityError::UnknownToken(String::new()));
    assert_eq!(
        error.token(),
        "contents:bogus",
        "unknown tokens must fail closed and name the rejected token"
    );

    println!("Authority::parse(\"contents:write\") returns Ok(Authority::ContentsWrite)");
    println!(
        "Authority::parse(\"administration:write\") returns Ok(Authority::AdministrationWrite)"
    );
    println!("Authority::parse(\"contents:bogus\") returns Err(ParseAuthorityError)");
}

#[test]
fn authority_parse_unknown_errors() {
    assert!(
        matches!(
            Authority::parse("contents:bogus"),
            Err(ParseAuthorityError::UnknownToken(token)) if token == "contents:bogus"
        ),
        "unknown permission tokens must return ParseAuthorityError"
    );
}

#[test]
fn desugar_write_excludes_merge_admin() {
    let write = AuthoritySet::from_role("write")
        .unwrap_or_else(|err| panic!("write role should parse: {err}"));

    assert!(
        write.contains(Authority::ContentsWrite),
        "write role must contain contents:write"
    );
    assert!(
        write.contains(Authority::ReviewsWrite),
        "write role must contain reviews:write"
    );
    assert!(
        write.contains(Authority::PullRequestsWrite),
        "write role must contain pull_requests:write"
    );
    assert!(
        !write.contains(Authority::Merge),
        "write role must not include merge"
    );
    assert!(
        !write.contains(Authority::AdministrationWrite),
        "write role must not include administration:write"
    );

    println!("set contains Authority::ContentsWrite");
    println!("set contains Authority::ReviewsWrite");
    println!("set contains Authority::PullRequestsWrite");
    println!("set excludes Authority::Merge");
    println!("set excludes Authority::AdministrationWrite");
}

#[test]
fn desugar_admin_superuser_and_maintain() {
    let admin = AuthoritySet::from_role("admin")
        .unwrap_or_else(|err| panic!("admin role should parse: {err}"));
    assert_eq!(
        admin.len(),
        Authority::ALL.len(),
        "admin length must track Authority::ALL"
    );
    for authority in Authority::ALL {
        assert!(
            admin.contains(*authority),
            "admin must contain every Authority variant, missing {authority}"
        );
    }

    let maintain = AuthoritySet::from_role("maintain")
        .unwrap_or_else(|err| panic!("maintain role should parse: {err}"));
    assert!(
        maintain.contains(Authority::Merge),
        "maintain role must contain merge"
    );
    assert!(
        maintain.contains(Authority::AdministrationRead),
        "maintain role must contain administration:read"
    );
    assert!(
        !maintain.contains(Authority::AdministrationWrite),
        "maintain role must not contain administration:write"
    );

    println!("set contains Authority::Merge");
    println!("set contains Authority::AdministrationWrite");
    println!("EVERY Authority variant is present by iterating Authority::ALL");
    println!("admin set length == Authority::ALL length");
    println!("set contains Authority::AdministrationRead");
    println!("set excludes Authority::AdministrationWrite");
}

#[test]
fn list_loads_without_role() {
    let parsed = AuthoritySet::parse(["contents:write", "reviews:write"])
        .unwrap_or_else(|err| panic!("raw authority list should parse: {err}"));

    assert!(
        parsed.contains(Authority::ContentsWrite),
        "raw list must contain contents:write"
    );
    assert!(
        parsed.contains(Authority::ReviewsWrite),
        "raw list must contain reviews:write"
    );
    assert_eq!(
        parsed.len(),
        2,
        "raw list must load exactly the provided authorities"
    );

    println!("set contains Authority::ContentsWrite");
    println!("set contains Authority::ReviewsWrite");
    println!("set length == 2");
}

#[test]
fn list_equals_role_write() {
    let parsed = AuthoritySet::parse([
        "metadata:read",
        "contents:read",
        "pull_requests:read",
        "contents:write",
        "pull_requests:write",
        "reviews:write",
        "comments:write",
        "statuses:write",
    ])
    .unwrap_or_else(|err| panic!("full write token list should parse: {err}"));
    let write = AuthoritySet::from_role("write")
        .unwrap_or_else(|err| panic!("write role should parse: {err}"));

    assert_eq!(
        parsed, write,
        "raw full write list and write role must resolve identically"
    );

    println!("AuthoritySet::parse(<full write token list>) == AuthoritySet::from_role(\"write\")");
}

#[test]
fn optional_role_desugar_preserves_role_behavior() -> Result<(), ParseAuthorityError> {
    let empty = AuthoritySet::from_optional_role(None)?;
    assert!(
        empty.is_empty(),
        "missing role must contribute no authorities"
    );

    let maintain = AuthoritySet::from_optional_role(Some("maintain"))?;
    assert_eq!(
        maintain,
        AuthoritySet::from_role("maintain")?,
        "optional role desugar must match the existing role catalog exactly"
    );

    assert!(
        matches!(
            AuthoritySet::from_optional_role(Some("bogus")),
            Err(ParseAuthorityError::UnknownRole(role)) if role == "bogus"
        ),
        "unknown optional role must fail with the existing UnknownRole error"
    );

    Ok(())
}
