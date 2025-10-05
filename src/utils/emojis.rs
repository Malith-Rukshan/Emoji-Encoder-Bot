pub const EMOJI_LIST: &[&str] = &[
    "😀", "😂", "🥰", "😎", "🤔", "👍", "👎", "👏",
    "😅", "🤝", "🎉", "🎂", "🍕", "❤️", "🌞", "🌙",
    "🔥", "💯", "🚀", "👀", "💀", "🥹",
];

/// Get a random emoji from the list
pub fn get_random_emoji() -> &'static str {
    use rand::Rng;
    let mut rng = rand::rng();
    let idx = rng.random_range(0..EMOJI_LIST.len());
    EMOJI_LIST[idx]
}
