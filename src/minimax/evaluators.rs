use def::GameState;

struct SingleSampleEvaluator<S, MS, TE>
    where S: GameState,
          MS: MoveSelector<S>,
          TE: TerminalEvaluator<S> {
  _s: PhantomData<S>,
  move_selector: MS,
  terminal_evaluator: TE,
  max_moves_ahead: i32
}

impl<S, MS, TE> SingleSampleEvaluator<S, MS, TE>
    where S: GameState,
          MS: MoveSelector<S>,
          TE: TerminalEvaluator<S> {
  fn new(ms: MS, e: TE) -> Self {
    SingleSampleEvaluator{
      _s: PhantomData::default(),
      move_selector: ms,
      terminal_evaluator: e,
      max_moves_ahead: 10000
    }
  }
}

impl<S, MS, E> Evaluator<S> for SingleSampleEvaluator<S, MS, E>
    where S: GameState,
          MS: MoveSelector<S>,
          E: TerminalEvaluator<S> {
  fn evaluate(&self, state: &S) -> f32 {
    let mut s: S = state.clone();
    let mut counter = 0;
    while counter < self.max_moves_ahead {
      counter += 1;
      match self.move_selector.select(&s) {
        Some(m) => s.apply(m).ok(),
        None => break
      };
    }

    if counter >= self.max_moves_ahead {
      println!("State with infinite sampling:\n{}", s);
      writeln!(&mut std::io::stderr(), "Counter: {}", counter).unwrap();
      panic!();
    } else {
      self.terminal_evaluator.evaluate_terminal(&s).unwrap()
    }
  }
}

struct SimpleRecursiveEvaluator<S, MG, HE> {
  _s: PhantomData<S>,
  move_generator: MG,
  heuristic_evaluator: HE,
  depth: i32
}

impl<S, MG, HE> SimpleRecursiveEvaluator<S, MG, HE>
  where S: GameState,
        MG: MoveGenerator<S>,
        HE: Evaluator<S> {
  pub fn new(mg: MG, e: HE, d: i32) -> Self {
    SimpleRecursiveEvaluator {
      _s: PhantomData::default(),
      move_generator: mg,
      heuristic_evaluator: e,
      depth: d
    }
  }

  fn evaluate_rec(&self, state: &S, d: i32) -> f32 {
    if d == 0 {
      return self.heuristic_evaluator.evaluate(state);
    }
    let moves = self.move_generator.generate(state);
    if moves.len() == 0 {
      return self.heuristic_evaluator.evaluate(state);
    }

    let mut best_score = WORST_SCORE;

    for &m in moves.iter() {
      let mut state_clone = state.clone();
      state_clone.apply(m).ok();
      let score = self.evaluate_rec(&state_clone, d - 1);
      if score > best_score {
        best_score = score;
      }
    }

    best_score
  }
}

impl<S, MG, HE> MoveSelector<S> for SimpleRecursiveEvaluator<S, MG, HE>
    where S: GameState,
          MG: MoveGenerator<S>,
          HE: Evaluator<S> {
  fn select(&self, state: &S) -> Option<S::Move> {
    let moves = self.move_generator.generate(state);
    if moves.len() == 0 { return None; }

    let evaluate_move = |m: S::Move| -> f32 {
      let mut state_clone = state.clone();
      state_clone.apply(m).ok();
      self.evaluate_rec(&state_clone, self.depth - 1)
    };

    let mut best_move: S::Move = moves[0];
    let mut best_score = evaluate_move(best_move);

    for m in moves.iter().skip(1) {
      let score = evaluate_move(*m);
      if score > best_score {
        best_move = *m;
        best_score = score;
      }
    }

    Some(best_move)
  }
}

impl <S, MG, HE> Evaluator<S> for SimpleRecursiveEvaluator<S, MG, HE>
  where S: GameState,
        MG: MoveGenerator<S>,
        HE: Evaluator<S> {
  fn evaluate(&self, state: &S) -> f32 {
    self.evaluate_rec(state, self.depth)
  }
}

struct MaxScoreSelector<S, MG, E> {
  _s: PhantomData<S>,
  move_generator: MG,
  evaluator: E
}

impl<S, MG, E> MaxScoreSelector<S, MG, E>
    where S: GameState,
          MG: MoveGenerator<S>,
          E: Evaluator<S> {
  pub fn new(mg: MG, e: E) -> Self {
    MaxScoreSelector {
      _s: PhantomData::default(),
      move_generator: mg,
      evaluator: e
    }
  }
}

impl<S, MG, E> MoveSelector<S> for MaxScoreSelector<S, MG, E>
    where S: GameState,
          MG: MoveGenerator<S>,
          E: Evaluator<S> {
  fn select(&self, state: &S) -> Option<S::Move> {
    let moves = self.move_generator.generate(state);
    if moves.len() == 0 { return None; }

    let mut best_move: S::Move = moves[0];
    let mut best_score = self.evaluator.evaluate_move(state, best_move);

    for m in moves.iter().skip(1) {
      let score = self.evaluator.evaluate_move(state, *m);
      if score > best_score {
        best_move = *m;
        best_score = score;
      }
    }

    Some(best_move)
  }
}

pub fn create_greedy_move_selector<'a, S, MG, DMS, TE>(
        move_generator: MG,
        default_move_selector: DMS,
        terminal_evaluator: TE) -> Box<MoveSelector<S> + 'a>
    where S: GameState + 'a,
          MG: MoveGenerator<S> + 'a,
          DMS: MoveSelector<S> + 'a,
          TE: TerminalEvaluator<S> + 'a {
  let ss_evaluator = SingleSampleEvaluator::new(
      default_move_selector, terminal_evaluator);
  let selector = MaxScoreSelector::new(move_generator, ss_evaluator);

  Box::new(selector)
}

pub fn create_recursive_selector<'a, S, MG, DMS, TE>(
        move_generator: MG,
        default_move_selector: DMS,
        terminal_evaluator: TE,
        depth: i32) -> Box<MoveSelector<S> + 'a>
    where S: GameState + 'a,
          MG: MoveGenerator<S> + 'a,
          DMS: MoveSelector<S> + 'a,
          TE: TerminalEvaluator<S> + 'a {
  if depth == 0 {
    return Box::new(default_move_selector);
  }
  let ss_evaluator = SingleSampleEvaluator::new(
      default_move_selector, terminal_evaluator);
  let selector = SimpleRecursiveEvaluator::new(
      move_generator, ss_evaluator, depth);

  Box::new(selector)
}
