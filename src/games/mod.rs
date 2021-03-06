mod gomoku;
mod hexapawn;
pub mod subtractor;

pub use self::gomoku::{Gomoku, GomokuLineFeatureExtractor};
pub use self::hexapawn::{Hexapawn, HexapawnCompleteExtractor, HexapawnNumberOfPawnsExtractor};
pub use self::subtractor::{Subtractor, SubtractorFeatureExtractor};

#[macro_export]
macro_rules! call_with_game {
  ($func:expr, $game_spec:expr, $( $arg:expr ),* ) => {
    match $game_spec {
      &$crate::spec::GameSpec::Gomoku => {
        $func($crate::games::Gomoku::default(), $( $arg ),*)
      },
      &$crate::spec::GameSpec::Hexapawn(width, height) => {
        $func($crate::games::Hexapawn::default(width, height), $( $arg ),*)
      },
      &$crate::spec::GameSpec::Subtractor(start, max_sub) => {
        $func($crate::games::Subtractor::default(start, max_sub), $( $arg ),*)
      },
    }
  }
}
