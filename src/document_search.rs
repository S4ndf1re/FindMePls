use probly_search::{score::bm25, FieldAccessor, Index, QueryResult, Tokenizer};
use std::{borrow::Cow, collections::HashSet, fmt::Debug, hash::Hash};

pub enum LimitOption {
    None,
    WordLengthBased,
    Limit(usize),
}

impl LimitOption {
    pub fn into_option(&self, s: &str) -> Option<usize> {
        match self {
            LimitOption::None => None,
            LimitOption::WordLengthBased => Some(levenshtein_distance_rule(s.len())),
            LimitOption::Limit(limit) => Some(*limit),
        }
    }
}

pub fn levenshtein_distance_rule(word_length: usize) -> usize {
    if word_length <= 3 {
        0
    } else if word_length <= 5 {
        1
    } else if word_length <= 7 {
        2
    } else if word_length <= 10 {
        3
    } else {
        4
    }
}

pub struct IndexEngine<K, D>
where
    K: Clone + Copy + Eq + Hash + Debug,
{
    bm25_search: Index<K>,
    tokenizer: Tokenizer,
    field_accessor: Vec<FieldAccessor<D>>,
    word_occurences: HashSet<String>,
}

impl<K, D> IndexEngine<K, D>
where
    K: Clone + Copy + Eq + Hash + Debug,
{
    pub fn new(
        fields_num: usize,
        field_accessor: Vec<FieldAccessor<D>>,
        tokenizer: Tokenizer,
    ) -> Self {
        Self {
            bm25_search: Index::new(fields_num),
            tokenizer,
            field_accessor,
            word_occurences: HashSet::new(),
        }
    }

    fn extract_words<'a>(&'a self, document: &'a D) -> Vec<Cow<'_, str>> {
        let mut words = Vec::new();

        for accessor in &self.field_accessor {
            let strings = accessor(document);
            for string in strings {
                let tokens = (self.tokenizer)(string);
                for token in tokens {
                    words.push(token);
                }
            }
        }
        words
    }

    pub fn index(&mut self, key: K, document: &D) {
        let words = self.extract_words(document);
        let mut word_list = HashSet::new();
        // TODO: filter all words that can be considered a filler word (the, a, an, etc.)
        for word in words {
            word_list.insert(word.into_owned());
        }

        for word in word_list {
            self.word_occurences.insert(word);
        }

        self.bm25_search
            .add_document(&self.field_accessor, self.tokenizer, key, document);
    }

    /// Find the best matching token for the given token.
    fn find_best_matching_autocorrect_token(
        &self,
        token: &Cow<'_, str>,
        limit: Option<usize>,
    ) -> Vec<Cow<'_, str>> {
        let mut matches = Vec::new();

        for k in self.word_occurences.iter() {
            let levi_distance = distance::levenshtein(token.as_ref(), k);
            match limit {
                Some(limit) => {
                    if levi_distance <= limit {
                        matches.push(Cow::Borrowed(k.as_str()));
                    }
                }
                None => {
                    matches.push(Cow::Borrowed(k.as_str().into()));
                }
            }
        }
        matches
    }

    /// Find the best matching token for the given token.
    /// First, the query is tokenized using the passed tokenizer.
    /// Then, each token is compared to the index using the levenshtein distance.
    /// The autocorrect might add similar tokens to the query, that match a query token in the
    /// given limit.
    /// When no token matches a word in the already existing index in the given limit, the
    /// original token is used.
    pub fn find_best_matching_autocorrect<'a>(
        &'a self,
        query: &'a str,
        limit: LimitOption,
    ) -> Vec<Cow<'_, str>> {
        let mut matches = Vec::new();
        let tokens = (self.tokenizer)(query);

        for token in tokens {
            let limit = limit.into_option(token.as_ref());
            let mut token_matches = self.find_best_matching_autocorrect_token(&token, limit);
            matches.append(&mut token_matches);
            matches.push(token);
        }
        matches
    }

    /// query as is using the bm25 search index
    pub fn query(&self, query: &str, fields_boost: &[f64]) -> Vec<QueryResult<K>> {
        self.bm25_search
            .query(query, &mut bm25::new(), self.tokenizer, fields_boost)
    }

    /// query with autocorrect using the bm25 search index.
    ///
    pub fn query_with_autocorrect(
        &self,
        query: &str,
        fields_boost: &[f64],
        limit: LimitOption,
    ) -> Vec<QueryResult<K>> {
        let query = self.find_best_matching_autocorrect(query, limit).join(" ");
        self.query(&query, fields_boost)
    }

    pub fn remove_document(&mut self, key: K) {
        self.bm25_search.remove_document(key);
    }
}

impl<K, D> Debug for IndexEngine<K, D>
where
    K: Clone + Copy + Eq + Hash + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexEngine").finish()
    }
}
