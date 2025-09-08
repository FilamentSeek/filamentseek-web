use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use strum_macros::EnumIter;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Product {
    pub uuid: String,
    pub name: String,
    pub price: Cents,
    pub price_per_kg: Cents,
    pub url: String,
    pub material: FilamentMaterial,
    pub diameter: FilamentDiameter,
    pub weight: Grams,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cents(pub u32);

impl Cents {
    pub fn as_dollars(&self) -> f32 {
        self.0 as f32 / 100.0
    }

    pub fn from_dollars(dollars: f32) -> Self {
        Cents((dollars * 100.0).round() as u32)
    }
}

impl Display for Cents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${:.2}", self.as_dollars())
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum FilamentMaterial {
    PLA,
    PLAPlus,
    ABS,
    PETG,
    TPU,
    Nylon,
    PC,
    ASA,
    Unspecified,
    Other(String),
}

pub const KNOWN_MATERIALS: &[FilamentMaterial] = &[
    FilamentMaterial::PLA,
    FilamentMaterial::PLAPlus,
    FilamentMaterial::ABS,
    FilamentMaterial::PETG,
    FilamentMaterial::TPU,
    FilamentMaterial::Nylon,
    FilamentMaterial::PC,
    FilamentMaterial::ASA,
];

impl Display for FilamentMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilamentMaterial::PLA => write!(f, "PLA"),
            FilamentMaterial::PLAPlus => write!(f, "PLA+"),
            FilamentMaterial::ABS => write!(f, "ABS"),
            FilamentMaterial::PETG => write!(f, "PETG"),
            FilamentMaterial::TPU => write!(f, "TPU"),
            FilamentMaterial::Nylon => write!(f, "Nylon"),
            FilamentMaterial::PC => write!(f, "Polycarbonate"),
            FilamentMaterial::ASA => write!(f, "ASA"),
            FilamentMaterial::Unspecified => write!(f, "Unspecified"),
            FilamentMaterial::Other(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Grams(pub u16);

impl Grams {
    pub fn as_kg(self) -> f32 {
        self.0 as f32 / 1000.0
    }

    pub fn from_kg_string(s: &str) -> Self {
        let v = s.trim().trim_end_matches("kg").trim().replace(',', ".");
        let kg: f32 = v.parse().unwrap_or(0.0);
        Grams((kg * 1000.0).round() as u16)
    }
}

impl Display for Grams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 % 1000 == 0 {
            write!(f, "{} kg", self.as_kg())
        } else {
            write!(f, "{} g", self.0)
        }
    }
}

/// Filament diameter in hundredths of a millimeter (e.g. 175 = 1.75 mm)
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
#[serde(into = "u16", try_from = "u16")]
pub enum FilamentDiameter {
    D175,
    D285,
    Other(u16),
}

impl From<FilamentDiameter> for u16 {
    fn from(d: FilamentDiameter) -> Self {
        match d {
            FilamentDiameter::D175 => 175,
            FilamentDiameter::D285 => 285,
            FilamentDiameter::Other(x) => x,
        }
    }
}

impl TryFrom<u16> for FilamentDiameter {
    type Error = &'static str;
    fn try_from(v: u16) -> Result<Self, Self::Error> {
        Ok(match v {
            175 => FilamentDiameter::D175,
            285 => FilamentDiameter::D285,
            x => FilamentDiameter::Other(x),
        })
    }
}

impl FilamentDiameter {
    pub const D175_H: u16 = 175;
    pub const D285_H: u16 = 285;

    #[inline]
    pub fn hundredths(&self) -> u16 {
        match *self {
            FilamentDiameter::D175 => Self::D175_H,
            FilamentDiameter::D285 => Self::D285_H,
            FilamentDiameter::Other(h) => h,
        }
    }

    #[inline]
    pub fn from_hundredths(h: u16) -> Self {
        match h {
            Self::D175_H => FilamentDiameter::D175,
            Self::D285_H => FilamentDiameter::D285,
            x => FilamentDiameter::Other(x),
        }
    }

    pub fn mm(&self) -> f32 {
        self.hundredths() as f32 / 100.0
    }

    pub fn mm_string(&self) -> String {
        format!("{:.2}", self.mm())
    }

    pub fn from_mm_string(s: &str) -> Self {
        let v = s.trim().trim_end_matches("mm").trim().replace(',', ".");
        let mm: f32 = v.parse().unwrap_or(0.0);
        let h = (mm * 100.0).round() as u16;
        Self::from_hundredths(h)
    }
}

impl Display for FilamentDiameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} mm", self.mm())
    }
}
