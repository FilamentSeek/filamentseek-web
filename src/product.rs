use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};
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
    pub retailer: Retailer,
    pub retailer_product_id: String,
    pub color: FilamentColor,
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
#[serde(try_from = "String", into = "String")]
pub enum FilamentMaterial {
    PLA,
    PLAPlus,
    ABS,
    PETG,
    TPU,
    Nylon,
    PC,
    ASA,
    PCTG,
    Unspecified,
    Other(String),
}

impl FromStr for FilamentMaterial {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "PLA" => Self::PLA,
            "PLAPlus" => Self::PLAPlus,
            "ABS" => Self::ABS,
            "PETG" => Self::PETG,
            "TPU" => Self::TPU,
            "Nylon" => Self::Nylon,
            "PC" => Self::PC,
            "ASA" => Self::ASA,
            "PCTG" => Self::PCTG,
            "Unspecified" => Self::Unspecified,
            other => Self::Other(other.to_string()),
        })
    }
}

impl From<String> for FilamentMaterial {
    fn from(s: String) -> Self {
        FilamentMaterial::from_str(&s).unwrap()
    }
}

impl From<FilamentMaterial> for String {
    fn from(m: FilamentMaterial) -> String {
        m.to_string()
    }
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
    FilamentMaterial::PCTG,
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
            FilamentMaterial::PCTG => write!(f, "PCTG"),
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
#[serde(try_from = "String", into = "String")]
pub enum Retailer {
    Amazon,
    Other(String),
}

impl FromStr for Retailer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Amazon" => Self::Amazon,
            other => Self::Other(other.to_string()),
        })
    }
}

impl std::fmt::Display for Retailer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Amazon => write!(f, "Amazon"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl From<String> for Retailer {
    fn from(s: String) -> Self {
        Retailer::from_str(&s).unwrap()
    }
}

impl From<Retailer> for String {
    fn from(p: Retailer) -> String {
        p.to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
#[serde(try_from = "String", into = "String")]
#[derive(Default)]
pub enum FilamentColor {
    Red,
    Blue,
    Green,
    Black,
    White,
    Gray,
    Silver,
    Brown,
    Beige,
    Transparent,
    Yellow,
    Orange,
    Purple,
    Pink,
    Cyan,
    Magenta,
    Gold,
    Bronze,
    Copper,
    GlowInTheDark,
    Multicolor,
    #[default]
    Unspecified,
    Other(String),
}

pub const KNOWN_COLORS: &[FilamentColor] = &[
    FilamentColor::Black,
    FilamentColor::White,
    FilamentColor::Gray,
    FilamentColor::Silver,
    FilamentColor::Brown,
    FilamentColor::Beige,
    FilamentColor::Transparent,
    FilamentColor::Red,
    FilamentColor::Blue,
    FilamentColor::Green,
    FilamentColor::Yellow,
    FilamentColor::Orange,
    FilamentColor::Purple,
    FilamentColor::Pink,
    FilamentColor::Cyan,
    FilamentColor::Magenta,
    FilamentColor::Gold,
    FilamentColor::Bronze,
    FilamentColor::Copper,
    FilamentColor::GlowInTheDark,
    FilamentColor::Multicolor,
];

impl FromStr for FilamentColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Black" => Self::Black,
            "White" => Self::White,
            "Gray" => Self::Gray,
            "Silver" => Self::Silver,
            "Brown" => Self::Brown,
            "Beige" => Self::Beige,
            "Transparent" => Self::Transparent,
            "Red" => Self::Red,
            "Blue" => Self::Blue,
            "Green" => Self::Green,
            "Yellow" => Self::Yellow,
            "Orange" => Self::Orange,
            "Purple" => Self::Purple,
            "Pink" => Self::Pink,
            "Cyan" => Self::Cyan,
            "Magenta" => Self::Magenta,
            "Gold" => Self::Gold,
            "Bronze" => Self::Bronze,
            "Copper" => Self::Copper,
            "GlowInTheDark" => Self::GlowInTheDark,
            "Multicolor" => Self::Multicolor,
            "Unspecified" => Self::Unspecified,
            other => Self::Other(other.to_string()),
        })
    }
}

impl fmt::Display for FilamentColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Black => write!(f, "Black"),
            Self::White => write!(f, "White"),
            Self::Gray => write!(f, "Gray"),
            Self::Silver => write!(f, "Silver"),
            Self::Brown => write!(f, "Brown"),
            Self::Beige => write!(f, "Beige"),
            Self::Transparent => write!(f, "Transparent"),
            Self::Red => write!(f, "Red"),
            Self::Blue => write!(f, "Blue"),
            Self::Green => write!(f, "Green"),
            Self::Yellow => write!(f, "Yellow"),
            Self::Orange => write!(f, "Orange"),
            Self::Purple => write!(f, "Purple"),
            Self::Pink => write!(f, "Pink"),
            Self::Cyan => write!(f, "Cyan"),
            Self::Magenta => write!(f, "Magenta"),
            Self::Gold => write!(f, "Gold"),
            Self::Bronze => write!(f, "Bronze"),
            Self::Copper => write!(f, "Copper"),
            Self::GlowInTheDark => write!(f, "GlowInTheDark"),
            Self::Multicolor => write!(f, "Multicolor"),
            Self::Unspecified => write!(f, "Unspecified"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl From<String> for FilamentColor {
    fn from(s: String) -> Self {
        FilamentColor::from_str(&s).unwrap()
    }
}

impl From<FilamentColor> for String {
    fn from(c: FilamentColor) -> String {
        c.to_string()
    }
}

impl FilamentColor {
    pub fn hex(&self) -> &'static str {
        match self {
            Self::Black => "#808080",
            Self::White => "#FFFFFF",
            Self::Gray => "#808080",
            Self::Silver => "#C0C0C0",
            Self::Brown => "#8B4513",
            Self::Beige => "#F5F5DC",
            Self::Transparent => "#FFFFFF",
            Self::Red => "#FF0000",
            Self::Blue => "#0000FF",
            Self::Green => "#008000",
            Self::Yellow => "#FFFF00",
            Self::Orange => "#FFA500",
            Self::Purple => "#800080",
            Self::Pink => "#FFC0CB",
            Self::Cyan => "#00FFFF",
            Self::Magenta => "#FF00FF",
            Self::Gold => "#FFD700",
            Self::Bronze => "#CD7F32",
            Self::Copper => "#B87333",
            Self::GlowInTheDark => "#ADFF2F",
            Self::Multicolor => "#808080",
            Self::Unspecified => "#808080",
            Self::Other(_) => "#808080",
        }
    }
}
