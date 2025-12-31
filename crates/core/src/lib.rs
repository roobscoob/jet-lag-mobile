use crate::hide_and_seek::HideAndSeekGame;

pub mod hide_and_seek;
pub mod map;
pub mod resource;
pub mod transit;

pub enum Game {
    HideAndSeek(HideAndSeekGame),
}
