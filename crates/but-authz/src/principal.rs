use crate::AuthoritySet;

/// Stable identifier for a governed principal.
///
/// ```
/// use but_authz::PrincipalId;
///
/// assert_eq!(PrincipalId::new("rust-implementer").as_str(), "rust-implementer");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrincipalId(String);

impl PrincipalId {
    /// Create a principal identifier from an in-memory handle.
    ///
    /// ```
    /// use but_authz::PrincipalId;
    ///
    /// let id = PrincipalId::new("agent");
    /// assert_eq!(id.as_str(), "agent");
    /// ```
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Return the principal identifier as a string slice.
    ///
    /// ```
    /// use but_authz::PrincipalId;
    ///
    /// assert_eq!(PrincipalId::new("agent").as_str(), "agent");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable identifier for a governed group.
///
/// ```
/// use but_authz::GroupName;
///
/// assert_eq!(GroupName::new("maintainers").as_str(), "maintainers");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupName(String);

impl GroupName {
    /// Create a group name.
    ///
    /// ```
    /// use but_authz::GroupName;
    ///
    /// let name = GroupName::new("code-reviewers");
    /// assert_eq!(name.as_str(), "code-reviewers");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Return the group name as a string slice.
    ///
    /// ```
    /// use but_authz::GroupName;
    ///
    /// assert_eq!(GroupName::new("maintainers").as_str(), "maintainers");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A governed actor with direct authorities and group memberships.
///
/// ```
/// use but_authz::{AuthoritySet, Principal, PrincipalId};
///
/// let principal = Principal::new(PrincipalId::new("rust-implementer"), AuthoritySet::empty(), []);
/// assert_eq!(principal.id().as_str(), "rust-implementer");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    id: PrincipalId,
    authorities: AuthoritySet,
    groups: Vec<GroupName>,
}

impl Principal {
    /// Create a principal from direct authorities and group memberships.
    ///
    /// ```
    /// use but_authz::{AuthoritySet, Principal, PrincipalId};
    ///
    /// let principal = Principal::new(PrincipalId::new("agent"), AuthoritySet::empty(), []);
    /// assert!(principal.authorities().is_empty());
    /// ```
    pub fn new<I>(id: PrincipalId, authorities: AuthoritySet, groups: I) -> Self
    where
        I: IntoIterator<Item = GroupName>,
    {
        Self {
            id,
            authorities,
            groups: groups.into_iter().collect(),
        }
    }

    /// Return the principal identifier.
    ///
    /// ```
    /// use but_authz::{AuthoritySet, Principal, PrincipalId};
    ///
    /// let principal = Principal::new(PrincipalId::new("agent"), AuthoritySet::empty(), []);
    /// assert_eq!(principal.id().as_str(), "agent");
    /// ```
    pub fn id(&self) -> &PrincipalId {
        &self.id
    }

    /// Return the principal's direct authority set.
    ///
    /// ```
    /// use but_authz::{AuthoritySet, Principal, PrincipalId};
    ///
    /// let principal = Principal::new(PrincipalId::new("agent"), AuthoritySet::empty(), []);
    /// assert!(principal.authorities().is_empty());
    /// ```
    pub fn authorities(&self) -> &AuthoritySet {
        &self.authorities
    }

    /// Return the groups this principal belongs to.
    ///
    /// ```
    /// use but_authz::{AuthoritySet, Principal, PrincipalId};
    ///
    /// let principal = Principal::new(PrincipalId::new("agent"), AuthoritySet::empty(), []);
    /// assert!(principal.groups().is_empty());
    /// ```
    pub fn groups(&self) -> &[GroupName] {
        &self.groups
    }
}

/// A governed group with authorities and principal membership.
///
/// ```
/// use but_authz::{AuthoritySet, Group, GroupName};
///
/// let group = Group::new(GroupName::new("maintainers"), AuthoritySet::empty(), []);
/// assert_eq!(group.name().as_str(), "maintainers");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    name: GroupName,
    authorities: AuthoritySet,
    members: Vec<PrincipalId>,
}

impl Group {
    /// Create a group from granted authorities and members.
    ///
    /// ```
    /// use but_authz::{AuthoritySet, Group, GroupName};
    ///
    /// let group = Group::new(GroupName::new("maintainers"), AuthoritySet::empty(), []);
    /// assert!(group.members().is_empty());
    /// ```
    pub fn new<I>(name: GroupName, authorities: AuthoritySet, members: I) -> Self
    where
        I: IntoIterator<Item = PrincipalId>,
    {
        Self {
            name,
            authorities,
            members: members.into_iter().collect(),
        }
    }

    /// Return the group name.
    ///
    /// ```
    /// use but_authz::{AuthoritySet, Group, GroupName};
    ///
    /// let group = Group::new(GroupName::new("maintainers"), AuthoritySet::empty(), []);
    /// assert_eq!(group.name().as_str(), "maintainers");
    /// ```
    pub fn name(&self) -> &GroupName {
        &self.name
    }

    /// Return the group's granted authority set.
    ///
    /// ```
    /// use but_authz::{AuthoritySet, Group, GroupName};
    ///
    /// let group = Group::new(GroupName::new("maintainers"), AuthoritySet::empty(), []);
    /// assert!(group.authorities().is_empty());
    /// ```
    pub fn authorities(&self) -> &AuthoritySet {
        &self.authorities
    }

    /// Return the principals that belong to this group.
    ///
    /// ```
    /// use but_authz::{AuthoritySet, Group, GroupName};
    ///
    /// let group = Group::new(GroupName::new("maintainers"), AuthoritySet::empty(), []);
    /// assert!(group.members().is_empty());
    /// ```
    pub fn members(&self) -> &[PrincipalId] {
        &self.members
    }
}
