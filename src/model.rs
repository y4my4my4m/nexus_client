use ratatui::style::Color;

// --- Data Structures ---

#[derive(Debug)]
pub struct Forum {
    pub name: String,
    pub description: String,
    pub threads: Vec<Thread>,
}

#[derive(Debug)]
pub struct ChatMessage {
    pub author: String,
    pub content: String,
    pub color: Color,
}

// Remove local Thread and Post definitions, use common::{Thread, Post}
pub use common::{Thread, Post, DirectMessage};

// // --- Mock Data Creation ---

// pub fn create_mock_forums() -> Vec<Forum> {
//     vec![
//         Forum {
//             name: "Decompiling Corporate ICE".to_string(),
//             description: "Tips and tricks for getting past the big boys' security.".to_string(),
//             threads: vec![
//                 Thread {
//                     title: "Militech's 'Aegis' Firewall - Any exploits?".to_string(),
//                     author: "jack_h.k".to_string(),
//                     posts: vec![
//                         Post { author: "jack_h.k".to_string(), content: "I've been probing their new Aegis system. It's tough. The outer layer seems to use quantum entanglement for key generation. Standard brute-forcing is useless.".to_string(), timestamp: 1633072800 },
//                         Post { author: "DataWitch".to_string(), content: "Heard that. You need to look for social engineering vectors. The human element is always the weakest link. Check their janitorial staff's public data.".to_string(), timestamp: 1633076400 },
//                     ],
//                     timestamp: 1633080000,
//                 },
//                 Thread {
//                     title: "Is Arasaka using AI for counter-intrusion?".to_string(),
//                     author: "n0de_r-unner".to_string(),
//                     posts: vec![
//                         Post { author: "n0de_r-unner".to_string(), content: "Every time I get close to their mainframe, the system adapts. It's not just scripts; it feels like it's learning. Anyone else seeing this?".to_string(), timestamp: 1633083600 },
//                     ],
//                     timestamp: 1633087200,
//                 },
//             ],
//         },
//         Forum {
//             name: "Black Market Bazaar".to_string(),
//             description: "Trade gear, software, and information. No feds.".to_string(),
//             threads: vec![
//                 Thread {
//                     title: "[WTS] Kiroshi Optics (Gen 3)".to_string(),
//                     author: "fixer_x".to_string(),
//                     posts: vec![
//                         Post { author: "fixer_x".to_string(), content: "Got a fresh pair, clean serial. 5000 eddies. No lowballers, I know what I have.".to_string(), timestamp: 1633090800 },
//                     ],
//                     timestamp: 1633094400,
//                 },
//             ],
//         },
//         Forum {
//             name: "General Discussion".to_string(),
//             description: "Anything that doesn't fit elsewhere.".to_string(),
//             threads: vec![
//                  Thread {
//                     title: "That new synth-pop band 'Chrome Dreams'".to_string(),
//                     author: "audiophile".to_string(),
//                     posts: vec![
//                         Post { author: "audiophile".to_string(), content: "Their latest album is a banger. The way they mix synth with feedback from malfunctioning cybernetics is art.".to_string(), timestamp: 1633098000 },
//                         Post { author: "Cyph3r".to_string(), content: "It's all corporate noise, man. Wake up.".to_string(), timestamp: 1633101600 },
//                     ],
//                     timestamp: 1633105200,
//                 },
//             ],
//         },
//     ]
// }