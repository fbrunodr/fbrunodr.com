#include <bits/stdc++.h>
using namespace std;


using str = string;
template <typename T>
using vec = vector<T>;


vec<str> dictionary;

// ------------------------------------------------------------------
// getFeedbackMask(guess, target)
// ------------------------------------------------------------------
// Compute Wordle feedback for `guess` against `target` using official rules
// (greens first, then yellows up to remaining letter counts; multiplicities handled).
//
// Encoding (10-bit mask packed in an int):
//   For each position i = 0..4 we use *two* bits:
//       bit (2*i)     -> YELLOW  (1 = letter present elsewhere; 0 = not marked yellow)
//       bit (2*i + 1) -> GREEN   (1 = correct letter in correct position)
//   Interpretation priority: if GREEN bit is set, ignore the YELLOW bit for that pos.
//   Thus each position has 3 logical states:
//       00 -> GRAY   (letter not present in remaining unmatched letters of target)
//       10 -> GREEN  (correct letter, correct place)
//       01 -> YELLOW (letter occurs in target but in a different position)
//       11 should not occur (we never set YELLOW when GREEN is set).
//
// Layout example (little-endian bit numbering):
//   i=0 uses bits [1:0], i=1 uses [3:2], ..., i=4 uses [9:8].
//   So mask fits in the low 10 bits of the int return value.
//
// Assumptions:
//   - guess.size() == 5, target.size() == 5
//   - all letters are lowercase 'a'..'z'
//   - target is a valid word from your dictionary
//
// Example usage:
//   int m = getFeedbackMask("crane", "cigar");
//   bool pos0_green  = m & (1 << 1);     // position 0 green?
//   bool pos2_yellow = m & (1 << (2*2)); // position 2 yellow?
//
// ------------------------------------------------------------------
int getFeedbackMask(const str& guess, const str& target){
    int feedback = 0;
    int remaining[26] = {0};
    for(char c : target) remaining[c - 'a']++;

    // Pass 1: greens
    for(int i = 0; i < 5; ++i)
        if(guess[i] == target[i]){
            feedback |= (1 << (2 * i + 1));
            --remaining[guess[i] - 'a'];
        }

    // Pass 2: yellows
    for(int i = 0; i < 5; ++i){
        if(feedback & (1 << (2 * i + 1))) continue;
        int letter = guess[i] - 'a';
        if(remaining[letter] > 0){
            feedback |= (1 << (2 * i));
            --remaining[letter];
        } // else stays '0'
    }
    return feedback;
}


inline bool isConsistent(const str& candidate, const str& guess, int feedback){
    return getFeedbackMask(guess, candidate) == feedback;
}


vec<str> filterWords(const vec<str>& currWords, const vec<pair<str,int>>& history){
    vec<str> filteredWords;
    copy_if(currWords.begin(), currWords.end(), back_inserter(filteredWords), [&](const str& candidate){
        return all_of(history.begin(), history.end(), [&](const auto& h){
            const auto& [guess, feedback] = h;
            return isConsistent(candidate, guess, feedback);
        });
    });
    return filteredWords;
}


vec<str> loadWords(){
    const char* path = "static/wordle_valid_words.txt";
    ifstream in(path);
    if(!in){
        cerr << "Error: could not open " << path << "\n";
        exit(1);
    }
    vec<str> words;
    str word;
    while(in >> word)
        words.push_back(word);
    return words;
}


vec<pair<str, double>> solver(const vec<str>& remaining){
    const int R = (int)remaining.size();
    if(R == 1) return {{remaining.front(), 0.0}};

    vec<pair<str, double>> guessesAndEntropies;
    unordered_set<str> remainingSet(remaining.begin(), remaining.end());

    for(const str& word : dictionary){
        vec<int> bucket(1024, 0);

        for(const str& target : remaining){
            int feedback = getFeedbackMask(word, target);
            bucket[feedback]++;
        }

        double H = 0.0;
        for(int size : bucket){
            if(size == 0) continue;
            double p = double(size) / double(R);
            H -= p * log2(p);
        }

        if(H > 0)
            guessesAndEntropies.emplace_back(word, H);
    }

    sort(guessesAndEntropies.begin(), guessesAndEntropies.end(), [&](const auto& a, const auto& b){
        if(a.second != b.second) return a.second > b.second;
        // words in remaining have higher priority given tie
        return remainingSet.count(a.first) > remainingSet.count(b.first);
    });

    return guessesAndEntropies;
}


int playGameAgainstTarget(
    const str &target,
    const str &startGuess,
    function<vec<pair<str, double>>(const vec<str>&)> solver
) {
    vec<pair<str, int>> history;

    str guess = startGuess;
    vec<str> remaining = dictionary;

    for (int move = 1; ; ++move){
        if (guess == target) return move;

        int feedback = getFeedbackMask(guess, target);
        history.emplace_back(guess, feedback);

        remaining = filterWords(remaining, history);
        assert(!remaining.empty());

        guess = solver(remaining).front().first;
    }
}


void evaluateAllStartWords(function<vec<pair<str, double>>(const vec<str>&)> solver){
    for(const auto& [startWord, _] : solver(dictionary)){ // start from the best words according to entropy
        int worst = 0;
        for(const str& target : dictionary){
            int moves = playGameAgainstTarget(target, startWord, solver);
            worst = max(worst, moves);
        }
        cout << startWord << " " << worst << endl;
    }
}


int feedbackToMask(const str& feedback){
    int mask = 0;
    for(int i = 0; i < 5; ++i)
        mask |= (feedback[i] - '0') << (2 * i);
    return mask;
}

// int main(){
//     dictionary = loadWords();
//     evaluateAllStartWords(solver);
//     return 0;
// }

int main(int argc, char* argv[]){
    dictionary = loadWords();

    if(argc % 2 != 1){
        cerr << "Usage: " << argv[0] << " <guess> <feedback> [...]\n";
        return 1;
    }

    vec<pair<str,int>> history;
    for(int i = 1; i < argc; i += 2){
        str guess    = argv[i];
        str feedback = argv[i + 1];
        history.emplace_back(guess, feedbackToMask(feedback));
    }

    vec<str> remaining = filterWords(dictionary, history);
    if(remaining.empty()){
        cerr << "No words match the given feedback" << endl;
        return 0;
    }

    vec<pair<str, double>> guessesAndEntropies = solver(remaining);
    cout << fixed << setprecision(4);
    for(const auto& [guess, entropy] : guessesAndEntropies)
        cout << entropy << " " << guess << endl;

    return 0;
}
