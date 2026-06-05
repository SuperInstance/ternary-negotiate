//! Ternary negotiation: agents negotiate using {-1=reject, 0=neutral, +1=accept} signals.

use std::collections::HashMap;

/// Negotiation stance
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stance { Reject, Neutral, Accept }

impl Stance {
    pub fn to_i8(self) -> i8 { match self { Stance::Reject => -1, Stance::Neutral => 0, Stance::Accept => 1 } }
    pub fn from_i8(v: i8) -> Self { match v { -1 => Stance::Reject, 0 => Stance::Neutral, 1 => Stance::Accept, _ => Stance::Neutral } }
}

/// An agent in a negotiation
#[derive(Clone, Debug)]
pub struct Negotiator {
    pub id: usize,
    pub stance: Stance,
    pub flexibility: f64, // 0-1: how much the agent moves toward consensus
    pub min_acceptable: i8, // minimum outcome this agent accepts
    pub history: Vec<Stance>,
}

impl Negotiator {
    pub fn new(id: usize, stance: Stance, flexibility: f64) -> Self {
        Self { id, stance, flexibility, min_acceptable: 0, history: vec![stance] }
    }

    /// Update stance based on group pressure
    pub fn respond(&mut self, group_avg: i8) -> Stance {
        let current = self.stance.to_i8() as f64;
        let target = current + self.flexibility * (group_avg as f64 - current);
        let new_val = target.round() as i8;
        self.stance = Stance::from_i8(new_val.clamp(-1, 1));
        self.history.push(self.stance);
        self.stance
    }
}

/// A negotiation session
pub struct Negotiation {
    pub agents: Vec<Negotiator>,
    pub rounds: usize,
    pub consensus_threshold: f64, // fraction needed for agreement
}

impl Negotiation {
    pub fn new(agents: Vec<Negotiator>, threshold: f64) -> Self {
        Self { agents, rounds: 0, consensus_threshold: threshold }
    }

    /// Run one round of negotiation
    pub fn round(&mut self) -> (i8, bool) {
        let avg = self.group_average();
        for agent in &mut self.agents {
            agent.respond(avg);
        }
        self.rounds += 1;
        let consensus = self.check_consensus();
        (avg, consensus)
    }

    pub fn group_average(&self) -> i8 {
        let sum: f64 = self.agents.iter().map(|a| a.stance.to_i8() as f64).sum();
        (sum / self.agents.len() as f64).round() as i8
    }

    pub fn check_consensus(&self) -> bool {
        let accepting = self.agents.iter().filter(|a| a.stance == Stance::Accept).count();
        (accepting as f64 / self.agents.len() as f64) >= self.consensus_threshold
    }

    /// Run until consensus or max rounds
    pub fn run_to_consensus(&mut self, max_rounds: usize) -> Option<(i8, usize)> {
        for _ in 0..max_rounds {
            let (_, consensus) = self.round();
            if consensus {
                return Some((self.group_average(), self.rounds));
            }
        }
        None
    }
}

/// Bilateral (two-party) negotiation with offers and counteroffers
pub struct BilateralNegotiation {
    pub party_a: Negotiator,
    pub party_b: Negotiator,
    pub current_offer: i8,
    pub midpoint: i8,
}

impl BilateralNegotiation {
    pub fn new(a: Negotiator, b: Negotiator) -> Self {
        let midpoint = ((a.stance.to_i8() + b.stance.to_i8()) as f64 / 2.0).round() as i8;
        Self { party_a: a, party_b: b, current_offer: midpoint, midpoint }
    }

    /// One round: each party moves toward midpoint
    pub fn exchange(&mut self) -> (i8, bool) {
        let a_stance = self.party_a.stance.to_i8();
        let b_stance = self.party_b.stance.to_i8();
        if a_stance == b_stance { return (a_stance, true); }
        
        let midpoint = ((a_stance as f64 + b_stance as f64) / 2.0).round() as i8;
        self.party_a.respond(midpoint);
        self.party_b.respond(midpoint);
        self.current_offer = midpoint;
        
        let agreed = self.party_a.stance == self.party_b.stance;
        (self.current_offer, agreed)
    }

    /// Run until agreement or stalemate
    pub fn negotiate(&mut self, max_rounds: usize) -> Option<i8> {
        for _ in 0..max_rounds {
            let (_, agreed) = self.exchange();
            if agreed { return Some(self.party_a.stance.to_i8()); }
        }
        None
    }
}

/// Voting-based negotiation
pub struct TernaryVote {
    pub proposals: Vec<i8>,
    pub votes: HashMap<usize, Vec<i8>>,
}

impl TernaryVote {
    pub fn new() -> Self { Self { proposals: Vec::new(), votes: HashMap::new() } }

    pub fn add_proposal(&mut self, value: i8) -> usize {
        self.proposals.push(value);
        self.proposals.len() - 1
    }

    pub fn vote(&mut self, voter_id: usize, proposal_id: usize, vote: i8) {
        self.votes.entry(voter_id).or_default().push(vote);
    }

    /// Tally: weighted sum of votes per proposal
    pub fn tally(&self) -> Vec<f64> {
        self.proposals.iter().enumerate().map(|(pid, _)| {
            let mut sum = 0.0;
            let mut count = 0;
            for votes in self.votes.values() {
                if let Some(&v) = votes.get(pid) {
                    sum += v as f64;
                    count += 1;
                }
            }
            if count > 0 { sum / count as f64 } else { 0.0 }
        }).collect()
    }

    pub fn winner(&self) -> Option<usize> {
        let scores = self.tally();
        scores.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stance_conversion() {
        assert_eq!(Stance::from_i8(-1), Stance::Reject);
        assert_eq!(Stance::from_i8(0), Stance::Neutral);
        assert_eq!(Stance::from_i8(1), Stance::Accept);
    }

    #[test]
    fn test_negotiator_responds() {
        let mut n = Negotiator::new(0, Stance::Accept, 0.5);
        n.respond(-1); // group is rejecting
        assert!(n.stance.to_i8() < 1); // should move toward reject
    }

    #[test]
    fn test_group_consensus() {
        let agents = vec![
            Negotiator::new(0, Stance::Accept, 0.8),
            Negotiator::new(1, Stance::Accept, 0.8),
            Negotiator::new(2, Stance::Accept, 0.8),
        ];
        let mut neg = Negotiation::new(agents, 0.67);
        assert!(neg.check_consensus());
    }

    #[test]
    fn test_run_to_consensus() {
        let agents = vec![
            Negotiator::new(0, Stance::Accept, 0.9),
            Negotiator::new(1, Stance::Accept, 0.9),
            Negotiator::new(2, Stance::Neutral, 0.9),
        ];
        let mut neg = Negotiation::new(agents, 0.67);
        let result = neg.run_to_consensus(50);
        assert!(result.is_some());
    }

    #[test]
    fn test_bilateral_agreement() {
        let a = Negotiator::new(0, Stance::Accept, 0.5);
        let b = Negotiator::new(1, Stance::Neutral, 0.5);
        let mut bn = BilateralNegotiation::new(a, b);
        let result = bn.negotiate(20);
        assert!(result.is_some());
    }

    #[test]
    fn test_voting() {
        let mut v = TernaryVote::new();
        let p1 = v.add_proposal(1);
        let p2 = v.add_proposal(-1);
        v.vote(0, p1, 1); v.vote(0, p2, -1);
        v.vote(1, p1, 1); v.vote(1, p2, 0);
        v.vote(2, p1, 0); v.vote(2, p2, -1);
        assert_eq!(v.winner(), Some(0)); // proposal 0 has higher score
    }

    #[test]
    fn test_round_increments() {
        let agents = vec![Negotiator::new(0, Stance::Neutral, 0.1)];
        let mut neg = Negotiation::new(agents, 1.0);
        neg.round();
        neg.round();
        assert_eq!(neg.rounds, 2);
    }

    #[test]
    fn test_flexible_agents_converge() {
        let agents = vec![
            Negotiator::new(0, Stance::Accept, 1.0),
            Negotiator::new(1, Stance::Reject, 1.0),
        ];
        let mut neg = Negotiation::new(agents, 0.5);
        // Fully flexible agents should converge to neutral
        neg.round();
        let avg = neg.group_average();
        assert_eq!(avg, 0);
    }
}
