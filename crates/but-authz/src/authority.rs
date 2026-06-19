use std::collections::BTreeSet;

/// A functional permission in the GitButler governance catalog.
///
/// ```
/// use but_authz::Authority;
///
/// assert_eq!(Authority::ContentsWrite.name(), "contents:write");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Authority {
    /// Read repository metadata.
    MetadataRead,
    /// Read repository contents.
    ContentsRead,
    /// Write repository contents.
    ContentsWrite,
    /// Read pull request state.
    PullRequestsRead,
    /// Write pull request state.
    PullRequestsWrite,
    /// Submit or update reviews.
    ReviewsWrite,
    /// Write review or pull request comments.
    CommentsWrite,
    /// Merge reviewed changes through a governed action.
    Merge,
    /// Read status checks.
    StatusesRead,
    /// Write status checks.
    StatusesWrite,
    /// Read administration state.
    AdministrationRead,
    /// Write administration state.
    AdministrationWrite,
}

impl Authority {
    /// Every functional authority in deterministic catalog order.
    ///
    /// ```
    /// use but_authz::Authority;
    ///
    /// assert!(Authority::ALL.contains(&Authority::Merge));
    /// ```
    pub const ALL: &'static [Self] = &[
        Self::MetadataRead,
        Self::ContentsRead,
        Self::ContentsWrite,
        Self::PullRequestsRead,
        Self::PullRequestsWrite,
        Self::ReviewsWrite,
        Self::CommentsWrite,
        Self::Merge,
        Self::StatusesRead,
        Self::StatusesWrite,
        Self::AdministrationRead,
        Self::AdministrationWrite,
    ];

    /// Parse a functional authority token.
    ///
    /// ```
    /// use but_authz::Authority;
    ///
    /// assert_eq!(Authority::parse("merge"), Ok(Authority::Merge));
    /// assert!(Authority::parse("contents:bogus").is_err());
    /// ```
    pub fn parse(token: &str) -> Result<Self, ParseAuthorityError> {
        match token {
            "metadata:read" => Ok(Self::MetadataRead),
            "contents:read" => Ok(Self::ContentsRead),
            "contents:write" => Ok(Self::ContentsWrite),
            "pull_requests:read" => Ok(Self::PullRequestsRead),
            "pull_requests:write" => Ok(Self::PullRequestsWrite),
            "reviews:write" => Ok(Self::ReviewsWrite),
            "comments:write" => Ok(Self::CommentsWrite),
            "merge" => Ok(Self::Merge),
            "statuses:read" => Ok(Self::StatusesRead),
            "statuses:write" => Ok(Self::StatusesWrite),
            "administration:read" => Ok(Self::AdministrationRead),
            "administration:write" => Ok(Self::AdministrationWrite),
            unknown => Err(ParseAuthorityError::UnknownToken(unknown.to_owned())),
        }
    }

    /// Return the stable token for this authority.
    ///
    /// ```
    /// use but_authz::Authority;
    ///
    /// assert_eq!(Authority::AdministrationWrite.name(), "administration:write");
    /// ```
    pub fn name(self) -> &'static str {
        match self {
            Self::MetadataRead => "metadata:read",
            Self::ContentsRead => "contents:read",
            Self::ContentsWrite => "contents:write",
            Self::PullRequestsRead => "pull_requests:read",
            Self::PullRequestsWrite => "pull_requests:write",
            Self::ReviewsWrite => "reviews:write",
            Self::CommentsWrite => "comments:write",
            Self::Merge => "merge",
            Self::StatusesRead => "statuses:read",
            Self::StatusesWrite => "statuses:write",
            Self::AdministrationRead => "administration:read",
            Self::AdministrationWrite => "administration:write",
        }
    }
}

impl std::fmt::Display for Authority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// A typed parse failure for authority tokens and role presets.
///
/// ```
/// use but_authz::{Authority, ParseAuthorityError};
///
/// assert!(matches!(
///     Authority::parse("contents:bogus"),
///     Err(ParseAuthorityError::UnknownToken(_))
/// ));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseAuthorityError {
    /// The token is not part of the authority catalog.
    #[error("unknown authority token: {0}")]
    UnknownToken(String),
    /// The role preset is not part of the desugar table.
    #[error("unknown authority role: {0}")]
    UnknownRole(String),
}

impl ParseAuthorityError {
    /// Return the rejected token or role name.
    ///
    /// ```
    /// use but_authz::Authority;
    ///
    /// let error = Authority::parse("contents:bogus").err();
    /// assert_eq!(error.as_ref().map(ParseAuthorityError::token), Some("contents:bogus"));
    /// ```
    pub fn token(&self) -> &str {
        match self {
            Self::UnknownToken(token) | Self::UnknownRole(token) => token,
        }
    }
}

/// A deterministic set of functional authorities.
///
/// ```
/// use but_authz::{Authority, AuthoritySet};
///
/// # fn main() -> Result<(), but_authz::ParseAuthorityError> {
/// let set = AuthoritySet::parse(["contents:write"])?;
/// assert!(set.contains(Authority::ContentsWrite));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AuthoritySet {
    authorities: BTreeSet<Authority>,
}

impl AuthoritySet {
    /// Create an empty authority set.
    ///
    /// ```
    /// use but_authz::AuthoritySet;
    ///
    /// assert!(AuthoritySet::empty().is_empty());
    /// ```
    pub fn empty() -> Self {
        Self::default()
    }

    /// Parse an authority-token list into a flat functional set.
    ///
    /// ```
    /// use but_authz::{Authority, AuthoritySet};
    ///
    /// # fn main() -> Result<(), but_authz::ParseAuthorityError> {
    /// let set = AuthoritySet::parse(["contents:write", "reviews:write"])?;
    /// assert!(set.contains(Authority::ReviewsWrite));
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse<I, T>(tokens: I) -> Result<Self, ParseAuthorityError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let authorities = tokens
            .into_iter()
            .map(|token| Authority::parse(token.as_ref()))
            .collect::<Result<BTreeSet<_>, _>>()?;
        Ok(Self { authorities })
    }

    /// Desugar a named role preset into a flat functional set.
    ///
    /// ```
    /// use but_authz::{Authority, AuthoritySet};
    ///
    /// # fn main() -> Result<(), but_authz::ParseAuthorityError> {
    /// let set = AuthoritySet::from_role("maintain")?;
    /// assert!(set.contains(Authority::Merge));
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_role(role: &str) -> Result<Self, ParseAuthorityError> {
        match role {
            "read" => Ok(Self::from_catalog(READ_AUTHORITIES)),
            "triage" => Ok(Self::from_catalog(TRIAGE_AUTHORITIES)),
            "write" => Ok(Self::from_catalog(WRITE_AUTHORITIES)),
            "maintain" => Ok(Self::from_catalog(MAINTAIN_AUTHORITIES)),
            "admin" => Ok(Self::from_catalog(Authority::ALL)),
            unknown => Err(ParseAuthorityError::UnknownRole(unknown.to_owned())),
        }
    }

    /// Return a new set containing authorities from both inputs.
    ///
    /// ```
    /// use but_authz::{Authority, AuthoritySet};
    ///
    /// # fn main() -> Result<(), but_authz::ParseAuthorityError> {
    /// let contents = AuthoritySet::parse(["contents:write"])?;
    /// let reviews = AuthoritySet::parse(["reviews:write"])?;
    /// assert!(contents.union(&reviews).contains(Authority::ReviewsWrite));
    /// # Ok(())
    /// # }
    /// ```
    pub fn union(&self, other: &Self) -> Self {
        let authorities = self
            .authorities
            .union(&other.authorities)
            .copied()
            .collect();
        Self { authorities }
    }

    /// Return true when the set contains the requested authority.
    ///
    /// ```
    /// use but_authz::{Authority, AuthoritySet};
    ///
    /// # fn main() -> Result<(), but_authz::ParseAuthorityError> {
    /// let set = AuthoritySet::from_role("write")?;
    /// assert!(set.contains(Authority::ContentsWrite));
    /// # Ok(())
    /// # }
    /// ```
    pub fn contains(&self, authority: Authority) -> bool {
        self.authorities.contains(&authority)
    }

    /// Return the number of unique authorities in the set.
    ///
    /// ```
    /// use but_authz::AuthoritySet;
    ///
    /// # fn main() -> Result<(), but_authz::ParseAuthorityError> {
    /// assert_eq!(AuthoritySet::parse(["merge"])?.len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn len(&self) -> usize {
        self.authorities.len()
    }

    /// Return true when the set contains no authorities.
    ///
    /// ```
    /// use but_authz::AuthoritySet;
    ///
    /// assert!(AuthoritySet::empty().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.authorities.is_empty()
    }

    /// Iterate the authorities in deterministic order.
    ///
    /// ```
    /// use but_authz::AuthoritySet;
    ///
    /// # fn main() -> Result<(), but_authz::ParseAuthorityError> {
    /// let set = AuthoritySet::parse(["merge"])?;
    /// assert_eq!(set.iter().map(|authority| authority.name()).collect::<Vec<_>>(), vec!["merge"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = Authority> + '_ {
        self.authorities.iter().copied()
    }

    fn from_catalog(catalog: &[Authority]) -> Self {
        let authorities = catalog.iter().copied().collect();
        Self { authorities }
    }
}

const READ_AUTHORITIES: &[Authority] = &[
    Authority::MetadataRead,
    Authority::ContentsRead,
    Authority::PullRequestsRead,
];

const TRIAGE_AUTHORITIES: &[Authority] = &[
    Authority::MetadataRead,
    Authority::ContentsRead,
    Authority::PullRequestsRead,
    Authority::StatusesRead,
];

const WRITE_AUTHORITIES: &[Authority] = &[
    Authority::MetadataRead,
    Authority::ContentsRead,
    Authority::PullRequestsRead,
    Authority::ContentsWrite,
    Authority::PullRequestsWrite,
    Authority::ReviewsWrite,
    Authority::CommentsWrite,
    Authority::StatusesWrite,
];

const MAINTAIN_AUTHORITIES: &[Authority] = &[
    Authority::MetadataRead,
    Authority::ContentsRead,
    Authority::PullRequestsRead,
    Authority::ContentsWrite,
    Authority::PullRequestsWrite,
    Authority::ReviewsWrite,
    Authority::CommentsWrite,
    Authority::StatusesWrite,
    Authority::Merge,
    Authority::AdministrationRead,
];
