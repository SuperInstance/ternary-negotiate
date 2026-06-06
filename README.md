# ternary-negotiate

Multi-agent negotiation where every stance is reject, neutral, or accept — and consensus emerges from pressure.

## Why This Exists

Real negotiations aren't binary. "No" can mean "not yet," "yes" can mean "reluctantly," and most participants spend their time somewhere in the middle. Binary voting (yes/no) forces premature commitment. Continuous scoring (rate 1-10) introduces false precision. Three stances — `{reject, neutral, accept}` — match how people actually make decisions when the options are presented honestly.

This crate models negotiation as a dynamical system. Agents hold stances in `{−1, 0, +1}`, have a flexibility parameter (how much they move toward the group average), and update their position each round. The system converges to consensus, stalls in disagreement, or oscillates — depending on the initial positions and flexibility values.

Three negotiation modes are supported: **multiparty** (N agents converging toward group consensus), **bilateral** (two agents making offers and counteroffers toward a midpoint), and **voting** (ternary votes on discrete proposals, highest average score wins).

## Architecture

```
Negotiator {stance, flexibility, history}
    │
    ├── respond(group_avg) ──► move stance toward group average by flexibility
    │
    ▼
Negotiation (multiparty)
    ├── round() ──► (group_avg, consensus_reached)
    ├── run_to_consensus(max_rounds) ──► Option<(avg, rounds_taken)>
    └── consensus_threshold: f64 (fraction needed)

BilateralNegotiation (two-party)
    ├── exchange() ──► (current_offer, agreed)
    └── negotiate(max_rounds) ──► Option<agreed_value>

TernaryVote (proposal-based)
    ├── add_proposal(value) ──► proposal_id
    ├── vote(voter_id, proposal_id, vote) ──► record vote {-1, 0, +1}
    ├── tally() ──► Vec<f64> (average score per proposal)
    └── winner() ──► Option<proposal_id>
```

**Key types:**

- **`Stance`** — `Reject` (`−1`), `Neutral` (`0`), `Accept` (`+1`). The atomic unit of negotiation.
- **`Negotiator`** — an agent with stance, flexibility `[0, 1]`, minimum acceptable outcome, and stance history.
- **`Negotiation`** — multiparty negotiation session. Each round: compute group average, all agents respond. Consensus when ≥`consensus_threshold` fraction of agents accept.
- **`BilateralNegotiation`** — two-party negotiation. Each exchange: compute midpoint, both agents move toward it. Terminates when stances match.
- **`TernaryVote`** — proposal-based voting. Voters assign `{-1, 0, +1}` to each proposal. Winner is the proposal with highest average score.

## Usage

```rust
use ternary_negotiate::{Negotiator, Stance, Negotiation, BilateralNegotiation, TernaryVote};

// Multiparty negotiation
let agents = vec![
    Negotiator::new(0, Stance::Accept, 0.9),   // enthusiastic, flexible
    Negotiator::new(1, Stance::Accept, 0.9),
    Negotiator::new(2, Stance::Neutral, 0.5),  // on the fence
    Negotiator::new(3, Stance::Reject, 0.3),   // skeptical, stubborn
];
let mut neg = Negotiation::new(agents, 0.67); // need 67% acceptance

// Run rounds
let (avg, consensus) = neg.round();
if consensus {
    println!("Consensus reached in {} rounds!", neg.rounds);
}

// Or run until consensus
if let Some((final_avg, rounds)) = neg.run_to_consensus(50) {
    println!("Agreed at stance {} after {} rounds", final_avg, rounds);
}

// Bilateral negotiation
let a = Negotiator::new(0, Stance::Accept, 0.5);
let b = Negotiator::new(1, Stance::Neutral, 0.5);
let mut bilateral = BilateralNegotiation::new(a, b);

if let Some(agreement) = bilateral.negotiate(20) {
    println!("Two parties agreed at: {}", agreement);
}

// Ternary voting on proposals
let mut vote = TernaryVote::new();
let p1 = vote.add_proposal(1);   // proposal: accept
let p2 = vote.add_proposal(-1);  // proposal: reject

vote.vote(0, p1, 1);   // voter 0 accepts proposal 1
vote.vote(0, p2, -1);  // voter 0 rejects proposal 2
vote.vote(1, p1, 1);   // voter 1 accepts proposal 1
vote.vote(1, p2, 0);   // voter 1 neutral on proposal 2
vote.vote(2, p1, 0);   // voter 2 neutral on proposal 1
vote.vote(2, p2, -1);  // voter 2 rejects proposal 2

let scores = vote.tally(); // average scores per proposal
let winner = vote.winner(); // Some(0) — proposal 1 wins

// Check convergence of fully flexible agents
let agents = vec![
    Negotiator::new(0, Stance::Accept, 1.0),   // fully flexible
    Negotiator::new(1, Stance::Reject, 1.0),
];
let mut neg = Negotiation::new(agents, 0.5);
neg.round();
// Fully flexible agents converge to neutral (0) — the midpoint
```

## API Reference

### `Stance`

| Variant | Value |
|---------|-------|
| `Reject` | −1 |
| `Neutral` | 0 |
| `Accept` | +1 |

Methods: `.to_i8()`, `.from_i8(i8)`

### `Negotiator`

| Method | Description |
|--------|-------------|
| `Negotiator::new(id, stance, flexibility)` | Create agent. Flexibility in `[0, 1]`. |
| `.respond(group_avg)` | Update stance: move toward `group_avg` by `flexibility` factor. Returns new stance. |

Fields: `id: usize`, `stance: Stance`, `flexibility: f64`, `min_acceptable: i8`, `history: Vec<Stance>`

### `Negotiation`

| Method | Description |
|--------|-------------|
| `Negotiation::new(agents, threshold)` | Create session with consensus threshold (fraction 0-1) |
| `.round()` | Execute one round. Returns `(group_avg: i8, consensus: bool)`. |
| `.group_average()` | Current average stance |
| `.check_consensus()` | True if ≥threshold fraction of agents accept |
| `.run_to_consensus(max_rounds)` | Run until consensus or max rounds. Returns `Some((avg, rounds))`. |

### `BilateralNegotiation`

| Method | Description |
|--------|-------------|
| `BilateralNegotiation::new(a, b)` | Create two-party negotiation |
| `.exchange()` | One round: both move toward midpoint. Returns `(offer, agreed)`. |
| `.negotiate(max_rounds)` | Run until agreement or stalemate. Returns `Option<agreed_value>`. |

### `TernaryVote`

| Method | Description |
|--------|-------------|
| `TernaryVote::new()` | Empty vote session |
| `.add_proposal(value)` | Add proposal, return proposal id |
| `.vote(voter_id, proposal_id, vote)` | Record vote `{-1, 0, +1}` |
| `.tally()` | Average score per proposal |
| `.winner()` | Proposal id with highest score |

## The Deeper Idea

Negotiation as a dynamical system. Each agent is a state variable holding a ternary value. The update rule — move toward the group average weighted by flexibility — is a form of **Deffuant model** opinion dynamics, simplified to three states. The system's behavior depends on the flexibility distribution:

- **High flexibility** (all agents near 1.0): rapid convergence to the group mean. The system reaches consensus in a few rounds.
- **Low flexibility** (all agents near 0.0): stagnation. Nobody moves, no consensus.
- **Mixed flexibility**: the interesting case. Flexible agents absorb into the majority, stubborn agents hold out. If the stubborn agents are below the consensus threshold, they're overrun. If they're above it, they block consensus — a realistic model of veto power.

The bilateral negotiation is a **bisection in ternary space**. Two agents start at different stances, compute the midpoint, and move toward it. With non-zero flexibility, they converge in at most `log₂(span)` rounds — in ternary space, that's 1-2 rounds maximum. The interesting case is when flexibility < 1: agents move *toward* the midpoint but don't reach it, requiring multiple rounds.

Ternary voting with `{-1, 0, +1}` scores is equivalent to **approval voting with an explicit abstention option**. The `-1` is active disapproval (downvote), `0` is neutrality (abstain), `+1` is approval (upvote). The average score is the net approval rating. This is more expressive than binary approval voting because it distinguishes "didn't vote" from "voted against."

## Related Crates

- **`ternary-proof`** — verification with ternary verdicts, the security analogue of negotiation
- **`ternary-route`** — routing with accept/queue/reject, structurally parallel to negotiation outcomes
- **`ternary-scheduler`** — ternary priority scheduling, where negotiation resolves resource contention
