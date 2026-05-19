class UserProfile {
  final String userId;
  final String? displayName;
  final String? avatarUrl;

  const UserProfile({
    required this.userId,
    this.displayName,
    this.avatarUrl,
  });

  factory UserProfile.fromMap(Map<String, dynamic> map) {
    return UserProfile(
      userId: map['id']?.toString() ?? '',
      displayName: map['display_name'] as String?,
      avatarUrl: map['avatar_url'] as String?,
    );
  }
}
