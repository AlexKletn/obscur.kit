An incremental change to the semantic tree an adapter applies without a full
snapshot re-push (spec §7.1).

After the initial snapshot, adapters keep the platform accessibility tree in
sync by replaying a stream of these fine-grained updates rather than pushing
a fresh snapshot on every frame. Each variant names the primary affected
node (if any); [`affected`](Self::affected) extracts it and
[`is_structural`](Self::is_structural) distinguishes changes that reshape the
tree from pure content or focus changes.
