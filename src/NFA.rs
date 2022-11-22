use crate::{RegularExpression as RE, DisplayGraph::ToDisplayGraph};
use std::collections::BTreeMap;
use crate::DisplayGraph::*;
#[derive(Debug)]
pub struct NFA {
    num_states: usize,
    start_state: usize,
    end_states: Vec<usize>,
    transitions: Vec<BTreeMap<char, Vec<usize>>>,
}

impl ToDisplayGraph for NFA{
    fn to_display_graph(&self) -> DisplayGraph {
        let mut done=vec![false;self.num_states];
        let mut child =vec![];
        let mut graph=vec![];
        let mut labels=vec![];
        let mut edge:Vec<(usize,usize,Option<char>)>=Vec::new();
        graph.push(vec![0 as usize]);        
        child.push(0);
        done[0]=true;
        while !child.is_empty() {
            let mut current_nodes=vec![];
            let mut newchild =vec![];    
            
            for index in child{
                current_nodes.push(index);
                labels.push(index.to_string());

                for i in self.transitions[index].keys(){
                    for j in &self.transitions[index][i]{
                        edge.push((index,*j,Some(*i)));
                        if !done[*j] {
                            done[*j]=true;
                            newchild.push(*j);
                        }
                    }
                }

            }
            graph.push(current_nodes);
            child=newchild;
        }
        DisplayGraph::new(edge,labels,graph)
    }
}

impl NFA {
    fn new() -> Self {
        Self {
            num_states: 0,
            start_state: 0,
            end_states: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub fn from_regex(regex: &RE::ReOperator) -> Self {
        let mut nfa = Self::new();
        let (start, end) = nfa.recursive_from_regex(regex);
        nfa.start_state = start;
        nfa.end_states.push(end);
        nfa
    }

    fn recursive_from_regex(&mut self, regex: &RE::ReOperator) -> (usize, usize) {
        let add_state = |nfa: &mut NFA| {
            nfa.num_states += 1;
            nfa.transitions.push(BTreeMap::new());
            nfa.num_states - 1
        };  
        let add_start_end = |nfa: &mut NFA| {
            (add_state(nfa), add_state(nfa))
        };

        let (start,end) = match regex{
            RE::ReOperator::Concat(left, right) => {
                let (l_start,l_end) = self.recursive_from_regex(left);
                let (r_start,r_end) = self.recursive_from_regex(right);
                self.transitions[l_end].entry('ε').or_insert(Vec::new()).push(r_start);

                (l_start,r_end)
            },
            RE::ReOperator::Or(left, right) => {
                let (start,end) =add_start_end(self);

                let (l_start,l_end) = self.recursive_from_regex(left);
                let (r_start,r_end) = self.recursive_from_regex(right);

                self.transitions[start].entry('ε').or_insert(Vec::new()).push(l_start);
                self.transitions[start].entry('ε').or_insert(Vec::new()).push(r_start);
                self.transitions[r_end].entry('ε').or_insert(Vec::new()).push(end);
                self.transitions[l_end].entry('ε').or_insert(Vec::new()).push(end);

                (start,end)
            },
            RE::ReOperator::KleeneStar(inner) => {
                let (start,end) =add_start_end(self);
                let (i_start,i_end) = self.recursive_from_regex(inner);

                self.transitions[start].entry('ε').or_insert(Vec::new()).push(end);
                self.transitions[i_end].entry('ε').or_insert(Vec::new()).push(i_start);
                self.transitions[start].entry('ε').or_insert(Vec::new()).push(i_start);
                self.transitions[i_end].entry('ε').or_insert(Vec::new()).push(end);

                (start,end)
            },
            RE::ReOperator::Char(c) => {
                let (start,end) =add_start_end(self);
                self.transitions[start].entry(*c).or_insert(Vec::new()).push(end);

                (start,end)
            },
        };
        //nfa.transitions.push(BTreeMap::new());
        //nfa.transitions.push(BTreeMap::new());
        //nfa.transitions[nfa.start_state as usize].insert(regex.symbol, nfa.end_states[0]);
        (start,end)
    }
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn display_test(){
        let regex = RE::ReOperator::Or(
            Box::new(RE::ReOperator::Char('a')),
            Box::new(RE::ReOperator::Char('b')),
        );
        let nfa = NFA::from_regex(&regex);
        println!("{:?}",nfa);
    }
}
