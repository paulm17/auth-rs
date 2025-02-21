use rand::Rng;

pub fn expand_alphabet(pattern: &str) -> String {
  let mut result = String::new();
  let mut chars = pattern.chars().peekable();
  
  while let Some(c) = chars.next() {
    if chars.peek() == Some(&'-') {
      chars.next(); // consume the '-'
      if let Some(end) = chars.next() {
        for ch in c..=end {
          result.push(ch);
        }
      }
    } else {
      result.push(c);
    }
  }
  result
}

pub fn create_random_string_generator(base_patterns: &[&str]) -> impl Fn(usize, Option<&str>) -> String {
  let base_char_set: String = base_patterns.iter()
    .map(|&pattern| expand_alphabet(pattern))
    .collect();
  
  if base_char_set.is_empty() {
    panic!("No valid characters provided for random string generation.");
  }
  
  let base_chars: Vec<char> = base_char_set.chars().collect();
  
  move |length: usize, alphabet: Option<&str>| {
    if length == 0 {
      panic!("Length must be a positive integer.");
    }
    
    let chars = match alphabet {
      Some(pattern) => expand_alphabet(pattern).chars().collect::<Vec<_>>(),
      None => base_chars.clone(),
    };
      
    let char_set_len = chars.len();
    let mut rng = rand::thread_rng();
    
    (0..length)
      .map(|_| {
        let ch = chars[rng.gen_range(0..char_set_len)];
        // Randomly decide whether to make the character lowercase
        if rng.gen_range(0..2) == 0 {
          ch.to_ascii_lowercase()
        } else {
          ch
        }
      })
      .collect()
  }
}

// Usage example:
fn main() {
  let generate_random_string = create_random_string_generator(&["a-z", "0-9", "A-Z", "-_"]);
   
  // Generate a 32-character string using a custom alphabet
  let custom_random_string = generate_random_string(32, Some("A-Z0-9"));
  println!("auth token: {}", custom_random_string);
}