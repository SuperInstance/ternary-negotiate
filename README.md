# ternary-negotiate

Ternary negotiation protocols for multi-agent systems using {-1=reject, 0=neutral, +1=accept} signaling. Implements consensus dynamics, bilateral bargaining, and ternary voting with convergence guarantees.

## Why It Matters

Negotiation in multi-agent systems typically uses continuous-valued offers or binary accept/reject. Ternary negotiation adds the **neutral state** — a critical third option that represents abstention, uncertainty, or a willingness to defer. This prevents premature commitment, enables graceful concession, and models real-world diplomatic dynamics far more faithfully than binary protocols.

Applications:
- **Distributed consensus**: agents reaching agreement with bounded rationality
- **Automated bargaining**: resource allocation where parties have hard constraints
- **Voting systems**: ternary ballots (yes/no/abstain) with richer expressive power
- **Conflict resolution**: the neutral state acts as a cooling-off mechanism

## How It Works

### Agent Model

Each negotiator has a stance $s_i \in \{-1, 0, +1\}$ and flexibility $\phi_i \in [0, 1]$.

When exposed to group average $\bar{s}$, the agent updates via weighted linear interpolation:

$$s_i(t+1) = \text{round}\!\left(s_i(t) + \phi_i \cdot (\bar{s} - s_i(t))\right), \quad \text{clipped to } [-1, +1]$$

- $\phi_i = 0$: stubborn agent never moves
- $\phi_i = 1$: agent immediately adopts the group average

**Complexity:** O(k) per round for $k$ agents.

### Consensus Detection

Consensus is declared when the fraction of agents with $s_i = +1$ meets threshold $\tau$:

$$\frac{|\{i : s_i = +1\}|}{k} \geq \tau$$

### Bilateral Negotiation

Two parties alternate offers, each moving toward the midpoint:

$$m = \text{round}\!\left(\frac{s_a + s_b}{2}\right)$$

Both parties update toward $m$ using their flexibility parameter. Agreement occurs when $s_a = s_b$.

**Convergence:** if both parties have $\phi > 0$, they converge in at most $\lceil 1/\phi_{\min} \rceil$ rounds.

### Ternary Voting

Proposals are scored by weighted average vote:

$$\text{score}(p) = \frac{1}{n}\sum_{i=1}^{n} v_i(p), \quad v_i \in \{-1, 0, +1\}$$

The proposal with highest score wins. This naturally handles:
- **Unanimous support**: score = +1
- **Polarized opposition**: score ≈ 0 (cancels out)
- **Apathy**: many zeros drag the score toward neutral

## Quick Start

```rust
use ternary_negotiate::*;

// Multi-party consensus
let agents = vec![
    Negotiator::new(0, Stance::Accept, 0.8),
    Negotiator::new(1, Stance::Accept, 0.8),
    Negotiator::new(2, Stance::Neutral, 0.5),
    Negotiator::new(3, Stance::Reject, 0.6),
];
let mut neg = Negotiation::new(agents, 0.75); // 75% threshold
let result = neg.run_to_consensus(50);
// Agents converge toward majority position

// Bilateral negotiation
let a = Negotiator::new(0, Stance::Accept, 0.5);
let b = Negotiator::new(1, Stance::Reject, 0.5);
let mut bilateral = BilateralNegotiation::new(a, b);
let outcome = bilateral.negotiate(20);
// Returns Some(0) — they meet at neutral

// Ternary vote
let mut vote = TernaryVote::new();
let p1 = vote.add_proposal(1);  // positive proposal
let p2 = vote.add_proposal(-1); // negative proposal
vote.vote(0, p1, 1); vote.vote(0, p2, -1);
vote.vote(1, p1, 1); vote.vote(1, p2, 0);
assert_eq!(vote.winner(), Some(0));
```

## API

| Type / Function | Description |
|---|---|
| `Stance` | Enum: `Reject`, `Neutral`, `Accept` with `to_i8` / `from_i8` |
| `Negotiator::new(id, stance, flexibility)` | Create a negotiating agent |
| `Negotiation::new(agents, threshold)` | Multi-party negotiation session |
| `.round() → (i8, bool)` | Execute one round, return (avg, consensus_reached) |
| `.run_to_consensus(max_rounds) → Option<(i8, usize)>` | Run until consensus or timeout |
| `BilateralNegotiation::new(a, b)` | Two-party bargaining |
| `.negotiate(max_rounds) → Option<i8>` | Run until agreement or stalemate |
| `TernaryVote::new()` | Voting protocol |
| `.add_proposal(value) → usize`, `.vote(...)`, `.tally()`, `.winner()` | Voting operations |

## Architecture Notes

Negotiation dynamics are governed by the **γ + η = C** identity. Accept (+1) represents constructive alignment γ, Reject (-1) represents destructive opposition η, and Neutral (0) is the buffer state that prevents rigid polarization. The flexibility parameter $\phi$ controls how fast agents convert between γ and η populations.

The neutral state is essential for convergence: without it, agents with opposite stances (+1 and -1) can only flip abruptly, causing oscillation. With the neutral state, agents pass through zero smoothly, and the system converges to a stable fixed point where the sum of all stances is conserved within $C$.

## References

- Rubinstein, A. (1982). *Perfect Equilibrium in a Bargaining Model.* Econometrica, 50(1).
- Axelrod, R. (1997). *The Complexity of Cooperation.* Princeton University Press.
- Fatima, S. S. et al. (2014). *Negotiation and Decision Making.* MIT Press.
- Brandt, F. et al. (2016). *Handbook of Computational Social Choice.* Cambridge University Press.

## License

MIT
