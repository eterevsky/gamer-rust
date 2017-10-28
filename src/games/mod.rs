mod hexapawn;

pub use self::hexapawn::{Hexapawn, HexapawnNumberOfPawnsExtractor};

#[macro_export]
macro_rules! call_with_game {
  ($func:expr, $game_spec:expr, $( $arg:expr ),* ) => {
    match $game_spec {
      &$crate::spec::GameSpec::Gomoku => {
        $func($crate::gomoku::Gomoku::default(), $( $arg ),*)
      },
      &$crate::spec::GameSpec::Hexapawn(width, height) => {
        $func($crate::games::Hexapawn::default(width, height), $( $arg ),*)
      },
      &$crate::spec::GameSpec::Subtractor(start, max_sub) => {
        $func($crate::subtractor::Subtractor::default(start, max_sub), $( $arg ),*)
      },
    }
  }
}