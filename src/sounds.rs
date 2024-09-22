use macroquad::audio::{load_sound_from_bytes, Sound};

pub(crate) struct BallSounds {
    sounds: Vec<(&'static str, Vec<Sound>)>,
}

impl BallSounds {
    pub async fn new() -> Self {
        Self {
            sounds: vec![
                ("normal", get_normal().await),
                ("boing", get_boing().await),
                ("hatefilled", get_hatefilled().await),
            ],
        }
    }

    pub fn find(&self, current_string: &str) -> Option<(String, Vec<Sound>)> {
        if current_string.is_empty() {
            return None;
        }

        for (name, sounds) in self.sounds.iter() {
            if current_string.ends_with(name) {
                return Some((name.to_string(), sounds.clone()));
            }
        }
        None
    }

    pub fn get_first(&self) -> (String, Vec<Sound>) {
        let ball = &self.sounds[0];
        (ball.0.to_string(), ball.1.clone())
    }
}

async fn get_normal() -> Vec<Sound> {
    vec![
        load_sound_from_bytes(include_bytes!("../assets/bonk2.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/bonk3.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/bonk4.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/bonk5.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/bonk6.ogg"))
            .await
            .unwrap(),
    ]
}

async fn get_boing() -> Vec<Sound> {
    vec![
        load_sound_from_bytes(include_bytes!("../assets/boing/boing1.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/boing/boing2.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/boing/boing3.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/boing/boing4.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/boing/boing5.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/boing/boing6.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/boing/boing7.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/boing/boing8.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/boing/boing9.ogg"))
            .await
            .unwrap(),
    ]
}

async fn get_hatefilled() -> Vec<Sound> {
    vec![
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/Anvil1.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/Anvil2.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/Burn.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/Disgusted.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/Dont_touch_me.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/Excuse_Me.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/Fullofhate.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!(
            "../assets/hatefilled/I_Dont_Wanna_See_You.ogg"
        ))
        .await
        .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/I_HATE_YOU.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/Idon'tlikeyou.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/Ill.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/ImHateFilled.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!(
            "../assets/hatefilled/Jump_Off_Of_A_Microwave.ogg"
        ))
        .await
        .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/Knuckle.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/Ovenon.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!(
            "../assets/hatefilled/Whats_In_Your_Pocket_What_Did_You_Just_Put_In_Your_Pocket.ogg"
        ))
        .await
        .unwrap(),
        load_sound_from_bytes(include_bytes!(
            "../assets/hatefilled/Who_Do_You_Think_You_Are.ogg"
        ))
        .await
        .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/hatefilled/YOU'RE_A_LOSER.ogg"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!(
            "../assets/hatefilled/You_Would_Tell_Me_If_You_Stole_Something_Right.ogg"
        ))
        .await
        .unwrap(),
        load_sound_from_bytes(include_bytes!(
            "../assets/hatefilled/Youre_mentally_ill.ogg"
        ))
        .await
        .unwrap(),
    ]
}
