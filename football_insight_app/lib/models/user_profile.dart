class UserProfile {
  final int userId;
  final String accountIdentifier;
  final String? displayName;
  final String? avatarUrl;

  const UserProfile({
    required this.userId,
    required this.accountIdentifier,
    this.displayName,
    this.avatarUrl,
  });

  factory UserProfile.fromMap(Map<String, dynamic> map) {
    return UserProfile(
      userId: map['id'] as int,
      accountIdentifier: map['account_identifier'] as String,
      displayName: map['display_name'] as String?,
      avatarUrl: map['avatar_url'] as String?,
    );
  }
}
