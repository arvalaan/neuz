use std::{fmt, fs::File, time::Instant};

use serde::{Deserialize, Serialize};

use crate::data::Target;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotType {
    Unused,
    ChatMessage,
    Food,
    Pill,
    HealSkill,
    MpRestorer,
    FpRestorer,
    PickupPet,
    PickupMotion,
    AttackSkill,
    BuffSkill,
    RezSkill,
    Flying,
}
impl fmt::Display for SlotType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SlotType::Food => write!(f, "food"),
            SlotType::Pill => write!(f, "pill"),
            SlotType::MpRestorer => write!(f, "mp restorer"),
            SlotType::FpRestorer => write!(f, "fp restorer"),
            SlotType::PickupPet => write!(f, "pickup pet"),
            SlotType::AttackSkill => write!(f, "attack skill"),
            SlotType::BuffSkill => write!(f, "buff skill"),
            SlotType::RezSkill => write!(f, "rez skill"),
            SlotType::Flying => write!(f, "fly"),
            _ => write!(f, "??none??"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SlotBar {
    slots: Option<[Slot; 10]>,
}

impl Default for SlotBar {
    fn default() -> Self {
        Self {
            slots: Some([Slot::default(); 10]),
        }
    }
}

impl SlotBar {
    pub fn slots(&self) -> Vec<Slot> {
        return self.slots.unwrap().into_iter().collect::<Vec<_>>();
    }
    /// Get the first matching slot index
    pub fn get_slot_index(&self, slot_type: SlotType) -> Option<usize> {
        self.slots()
            .iter()
            .position(|slot| slot.slot_type == slot_type)
    }

    /// Get a random usable matching slot index
    pub fn get_usable_slot_index(
        &self,
        slot_type: SlotType,
        threshold: Option<u32>,
        last_slots_usage: [[Option<Instant>; 10]; 9],
        slot_bar_index: usize,
    ) -> Option<(usize, usize)> {
        self.slots()
            .iter()
            .enumerate()
            .filter(|(index, slot)| {
                slot.slot_type == slot_type
                    && slot.slot_enabled
                    && slot.slot_threshold.unwrap_or(100) >= threshold.unwrap_or(0)
                    && last_slots_usage[slot_bar_index][*index].is_none()
            })
            .min_by(|x, y| x.1.slot_threshold.cmp(&y.1.slot_threshold))
            //.choose(rng)
            .map(|(index, _)| (slot_bar_index, index))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Slot {
    slot_type: SlotType,
    slot_cooldown: Option<u32>,
    slot_threshold: Option<u32>,
    slot_enabled: bool,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            slot_type: SlotType::Unused,
            slot_cooldown: None,
            slot_threshold: None,
            slot_enabled: true,
        }
    }
}

impl Slot {
    pub fn get_slot_cooldown(&self) -> Option<u32> {
        let cooldown = self.slot_cooldown;
        if cooldown.is_some() {
            return cooldown;
        }
        return Some(100);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BotMode {
    Farming,
    Support,
}

impl ToString for BotMode {
    fn to_string(&self) -> String {
        match self {
            BotMode::Farming => "farming",
            BotMode::Support => "support",
        }
        .to_string()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FarmingConfig {
    /// Slot configuration
    slot_bars: Option<[SlotBar; 9]>,

    /// Search for mob circle pattern
    circle_pattern_rotation_duration: Option<u64>,

    /// Disable farming
    farming_enabled: Option<bool>,

    prevent_already_attacked: Option<bool>,

    is_manual_targetting: Option<bool>,

    obstacle_avoidance_max_try: Option<u32>,
}

impl FarmingConfig {

    pub fn circle_pattern_rotation_duration(&self) -> u64 {
        self.circle_pattern_rotation_duration.unwrap_or(30)
    }

    pub fn obstacle_avoidance_max_try(&self) -> u32 {
        self.obstacle_avoidance_max_try.unwrap_or(5)
    }

    pub fn slot_bars(&self) -> Vec<SlotBar> {
        self.slot_bars
            .map(|slots| slots.into_iter().collect::<Vec<_>>())
            .unwrap_or_else(|| [SlotBar::default(); 9].into_iter().collect::<Vec<_>>())
    }

    pub fn slots(&self, slot_bar_index: usize) -> Vec<Slot> {
        return self.slot_bars()[slot_bar_index].slots();
    }

    pub fn get_slot_cooldown(&self, slot_bar_index: usize, slot_index: usize) -> Option<u32> {
        return self.slots(slot_bar_index)[slot_index].get_slot_cooldown();
    }

    /// Get the first matching slot index
    pub fn slot_index(&self, slot_type: SlotType) -> Option<(usize, usize)> {
        for n in 0..9 {
            let found_index = self.slot_bars()[n].get_slot_index(slot_type);
            if found_index.is_some() {
                return Some((n, found_index.unwrap()));
            }
        }
        None
    }

    /// Get a random usable matching slot index
    pub fn get_usable_slot_index(
        &self,
        slot_type: SlotType,
        threshold: Option<u32>,
        last_slots_usage: [[Option<Instant>; 10]; 9],
    ) -> Option<(usize, usize)> {
        for n in 0..9 {
            let found_index = self.slot_bars()[n].get_usable_slot_index(
                slot_type,
                threshold,
                last_slots_usage,
                n,
            );
            if found_index.is_some() {
                return Some(found_index.unwrap());
            }
        }
        None
    }

    pub fn is_manual_targetting(&self) -> bool {
        self.is_manual_targetting.unwrap_or(false)
    }

    pub fn prevent_already_attacked(&self) -> bool {
        self.prevent_already_attacked.unwrap_or(true)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SupportConfig {
    slot_bars: Option<[SlotBar; 9]>,
}

impl SupportConfig {

    pub fn slot_bars(&self) -> Vec<SlotBar> {
        self.slot_bars
            .map(|slots| slots.into_iter().collect::<Vec<_>>())
            .unwrap_or_else(|| [SlotBar::default(); 9].into_iter().collect::<Vec<_>>())
    }

    pub fn slots(&self, slot_bar_index: usize) -> Vec<Slot> {
        return self.slot_bars()[slot_bar_index].slots();
    }

    pub fn get_slot_cooldown(&self, slot_bar_index: usize, slot_index: usize) -> Option<u32> {
        return self.slots(slot_bar_index)[slot_index].get_slot_cooldown();
    }

    /// Get a random usable matching slot index
    pub fn get_usable_slot_index(
        &self,
        slot_type: SlotType,
        threshold: Option<u32>,
        last_slots_usage: [[Option<Instant>; 10]; 9],
    ) -> Option<(usize, usize)> {
        for n in 0..9 {
            let found_index = self.slot_bars()[n].get_usable_slot_index(
                slot_type,
                threshold,
                last_slots_usage,
                n,
            );
            if found_index.is_some() {
                return Some(found_index.unwrap());
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// Change id to sync changes between frontend and backend
    change_id: u64,

    /// Whether the bot is running
    is_running: bool,

    /// The bot mode
    mode: Option<BotMode>,

    farming_config: FarmingConfig,
    support_config: SupportConfig,

    passive_mobs_colors: Option<[Option<u8>; 3]>,
    passive_tolerence: Option<u8>,
    aggressive_mobs_colors: Option<[Option<u8>; 3]>,
    aggressive_tolerence: Option<u8>,

    min_mobs_name_width: Option<u32>,
    max_mobs_name_width: Option<u32>,

    interval_between_buffs: Option<u64>,
    on_death_disconnect: Option<bool>,
    inactivity_timeout: Option<u64>,
    obstacle_avoidance_cooldown: Option<u64>,
    whitelist_enabled: Option<bool>,
    whitelist: Option<Vec<(u32, u32, String)>>

}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            change_id: 0,
            mode: None,
            is_running: false,
            farming_config: FarmingConfig::default(),
            support_config: SupportConfig::default(),

            passive_mobs_colors: Some([None, None, None]),
            passive_tolerence: Some(5),
            aggressive_mobs_colors: Some([None, None, None]),
            aggressive_tolerence: Some(10),

            min_mobs_name_width: Some(11),
            max_mobs_name_width: Some(180),

            interval_between_buffs:Some(2000),

            on_death_disconnect: Some(false),
            inactivity_timeout: Some(0),
            obstacle_avoidance_cooldown: Some(0),
            whitelist_enabled: Some(false),
            whitelist: None,
        }
    }
}

impl BotConfig {

    pub fn toggle_active(&mut self) {
        self.is_running = !self.is_running;
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn change_id(&self) -> u64 {
        self.change_id
    }

    pub fn changed(mut self) -> Self {
        self.change_id += 1;
        self
    }

    pub fn farming_config(&self) -> &FarmingConfig {
        &self.farming_config
    }

    pub fn support_config(&self) -> &SupportConfig {
        &self.support_config
    }

    pub fn mode(&self) -> Option<BotMode> {
        self.mode.clone()
    }

    pub fn min_mobs_name_width(&self) -> u32 {
        self.min_mobs_name_width.unwrap_or(11)
    }

    pub fn max_mobs_name_width(&self) -> u32 {
        self.max_mobs_name_width.unwrap_or(180)
    }

    pub fn passive_mobs_colors(&self) -> [Option<u8>; 3] {
        self.passive_mobs_colors.unwrap_or([None, None, None])
    }

    pub fn passive_tolerence(&self) -> u8 {
        self.passive_tolerence.unwrap_or(5)
    }

    pub fn aggressive_mobs_colors(&self) -> [Option<u8>; 3] {
        self.aggressive_mobs_colors.unwrap_or([None, None, None])
    }

    pub fn aggressive_tolerence(&self) -> u8 {
        self.aggressive_tolerence.unwrap_or(10)
    }

    pub fn interval_between_buffs(&self) -> u128 {
        self.interval_between_buffs.unwrap_or(2000).into()
    }

    pub fn on_death_disconnect(&self) -> bool {
        self.on_death_disconnect.unwrap_or(false)
    }

    pub fn inactivity_timeout(&self) -> u128 {
        self.inactivity_timeout.unwrap_or(0).into()
    }

    pub fn obstacle_avoidance_cooldown(&self) -> u128 {
        return self.obstacle_avoidance_cooldown.unwrap_or(0).into();
    }

    pub fn whitelist_enabled(&self) -> bool {
        return self.whitelist_enabled.unwrap_or(false);
    }

    pub fn match_whitelist(&self, target: Target) -> bool {
        if let Some(whitelist) = self.whitelist.clone() {
            let mut result = false;
            for mob in whitelist.iter() {
                if mob.0 == target.bounds.w && mob.1 == target.bounds.h {
                    result = true;
                    break;
                }
            };
            return result;
        } else {
            return true
        }

    }



    /// Serialize config to disk
    pub fn serialize(&self, path: String) {
        let config = {
            let mut config = self.clone();
            config.is_running = false;
            config
        };
        if let Ok(mut file) = File::create(path) {
            let _ = serde_json::to_writer(&mut file, &config);
        }
    }

    /// Deserialize config from disk
    pub fn deserialize_or_default(path: String) -> Self {
        if let Ok(mut file) = File::open(path) {
            serde_json::from_reader::<_, BotConfig>(&mut file).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}
