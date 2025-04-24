use serde::{Deserialize, Serialize};

// Import necessary types (assuming defined elsewhere)
use crate::models::placeholder::Placeholder;
use crate::models::text::TextContent;

use super::shape_properties::ShapeProperties; // TextContent will be defined below in text.rs

/// The type of a shape.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/shapes#Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShapeType {
    /// The shape type is unspecified.
    TypeUnspecified,
    /// Text box shape.
    TextBox,
    /// Rectangle shape.
    Rectangle,
    /// Round corner rectangle shape.
    RoundRectangle,
    /// Ellipse shape.
    Ellipse,
    /// Curved arc shape.
    Arc,
    /// Bent arrow shape.
    BentArrow,
    /// Bent up arrow shape.
    BentUpArrow,
    /// Bevel shape.
    Bevel,
    /// Block arc shape.
    BlockArc,
    /// Brace pair shape.
    BracePair,
    /// Bracket pair shape.
    BracketPair,
    /// Can shape.
    Can,
    /// Chevron shape.
    Chevron,
    /// Chord shape.
    Chord,
    /// Cloud shape.
    Cloud,
    /// Corner shape.
    Corner,
    /// Cube shape.
    Cube,
    /// Curved down arrow shape.
    CurvedDownArrow,
    /// Curved left arrow shape.
    CurvedLeftArrow,
    /// Curved right arrow shape.
    CurvedRightArrow,
    /// Curved up arrow shape.
    CurvedUpArrow,
    /// Decagon shape.
    Decagon,
    /// Diagonal stripe shape.
    DiagonalStripe,
    /// Diamond shape.
    Diamond,
    /// Dodecagon shape.
    Dodecagon,
    /// Donut shape.
    Donut,
    /// Double wave shape.
    DoubleWave,
    /// Down arrow shape.
    DownArrow,
    /// Callout down arrow shape.
    DownArrowCallout,
    /// Folded corner shape.
    FoldedCorner,
    /// Frame shape.
    Frame,
    /// Half frame shape.
    HalfFrame,
    /// Heart shape.
    Heart,
    /// Heptagon shape.
    Heptagon,
    /// Hexagon shape.
    Hexagon,
    /// Home plate shape.
    HomePlate,
    /// Horizontal scroll shape.
    HorizontalScroll,
    /// Irregular seal 1 shape.
    IrregularSeal1,
    /// Irregular seal 2 shape.
    IrregularSeal2,
    /// Left arrow shape.
    LeftArrow,
    /// Callout left arrow shape.
    LeftArrowCallout,
    /// Left brace shape.
    LeftBrace,
    /// Left bracket shape.
    LeftBracket,
    /// Left right arrow shape.
    LeftRightArrow,
    /// Callout left right arrow shape.
    LeftRightArrowCallout,
    /// Left right up arrow shape.
    LeftRightUpArrow,
    /// Left up arrow shape.
    LeftUpArrow,
    /// Lightning bolt shape.
    LightningBolt,
    /// Divide math shape.
    MathDivide,
    /// Equal math shape.
    MathEqual,
    /// Minus math shape.
    MathMinus,
    /// Multiply math shape.
    MathMultiply,
    /// Not equal math shape.
    MathNotEqual,
    /// Plus math shape.
    MathPlus,
    /// Moon shape.
    Moon,
    /// No smoking shape.
    NoSmoking,
    /// Non-isosceles trapezoid shape.
    NonIsoscelesTrapezoid,
    /// Notched right arrow shape.
    NotchedRightArrow,
    /// Octagon shape.
    Octagon,
    /// Parallelogram shape.
    Parallelogram,
    /// Pentagon shape.
    Pentagon,
    /// Pie shape.
    Pie,
    /// Plaque shape.
    Plaque,
    /// Plus shape.
    Plus,
    /// Quad arrow shape.
    QuadArrow,
    /// Callout quad arrow shape.
    QuadArrowCallout,
    /// Ribbon shape.
    Ribbon,
    /// Ribbon 2 shape.
    Ribbon2,
    /// Right arrow shape.
    RightArrow,
    /// Callout right arrow shape.
    RightArrowCallout,
    /// Right brace shape.
    RightBrace,
    /// Right bracket shape.
    RightBracket,
    /// Right triangle shape.
    RightTriangle,
    /// Round 1 rectangle shape.
    Round1Rectangle,
    /// Round 2 diagonal rectangle shape.
    Round2DiagonalRectangle,
    /// Round 2 same rectangle shape.
    Round2SameRectangle,
    /// Smiley face shape.
    SmileyFace,
    /// Snip 1 rectangle shape.
    Snip1Rectangle,
    /// Snip 2 diagonal rectangle shape.
    Snip2DiagonalRectangle,
    /// Snip 2 same rectangle shape.
    Snip2SameRectangle,
    /// Snip round rectangle shape.
    SnipRoundRectangle,
    /// Star 10 shape.
    Star10,
    /// Star 12 shape.
    Star12,
    /// Star 16 shape.
    Star16,
    /// Star 24 shape.
    Star24,
    /// Star 32 shape.
    Star32,
    /// Star 4 shape.
    Star4,
    /// Star 5 shape.
    Star5,
    /// Star 6 shape.
    Star6,
    /// Star 7 shape.
    Star7,
    /// Star 8 shape.
    Star8,
    /// Striped right arrow shape.
    StripedRightArrow,
    /// Sun shape.
    Sun,
    /// Trapezoid shape.
    Trapezoid,
    /// Triangle shape.
    Triangle,
    /// Up arrow shape.
    UpArrow,
    /// Callout up arrow shape.
    UpArrowCallout,
    /// Up down arrow shape.
    UpDownArrow,
    /// Callout up down arrow shape.
    UpDownArrowCallout,
    /// Uturn arrow shape.
    UturnArrow,
    /// Vertical scroll shape.
    VerticalScroll,
    /// Wave shape.
    Wave,
    /// Wedge ellipse callout shape.
    WedgeEllipseCallout,
    /// Wedge rectangle callout shape.
    WedgeRectangleCallout,
    /// Wedge round rectangle callout shape.
    WedgeRoundRectangleCallout,
    /// Speech bubble shape.
    Speech,
    /// Custom shape.
    Custom,
    // Note: The API lists many more shapes. This list captures those explicitly named in the enum documentation.
    // You might need to add more if you encounter them.
}

/// A PageElement kind representing a generic shape that doesn't have a more
/// specific classification.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#Shape
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shape {
    /// The type of the shape.
    pub shape_type: Option<ShapeType>,

    /// The text content of the shape.
    pub text: Option<TextContent>,

    /// The properties of the shape.
    pub shape_properties: Option<ShapeProperties>,

    /// The placeholder information for the shape. If set, the shape is a placeholder shape
    /// and inherits properties from the corresponding placeholder shape on the layout or master.
    pub placeholder: Option<Placeholder>,
}
