#include <bits/stdc++.h>
using namespace std;

int main(int argc, char* argv[]){
    if (argc < 3 || argc > 4) {
        cerr << "Usage: " << argv[0] << " <greens> <greys> [yellows]" << endl;
        return 1;
    }

    string greens = argv[1];
    string greys = argv[2];
    string yellows = (argc == 4) ? argv[3] : "";

    if(greens.size() != 5){
        cerr << "Greens must be 5 characters long" << endl;
        return 1;
    }

    ifstream f("./static/wordle_valid_words.txt");
    vector<string> words;
    {
        string word;
        while (f >> word)
            words.push_back(word);
    }

    vector<string> possible_words;
    for(auto& word : words){
        bool ok = true;
        for(int i = 0; i < 5; i++){
            if(greens[i] != '_' && greens[i] != word[i])
                ok = false;
        }
        for(char c : yellows){
            if(none_of(word.begin(), word.end(), [&](char c2) { return c2 == c; }))
                ok = false;
        }
        for(char c : greys){
            if(any_of(word.begin(), word.end(), [&](char c2) { return c2 == c; }))
                ok = false;
        }
        if(ok)
            possible_words.push_back(word);
    }

    vector<int> freq(128, 0);
    for(auto& word : possible_words)
        for(char c : word)
            freq[c]++;

    function<int(string)> score = [&](string word){
        int score = 0;
        set<char> used(word.begin(), word.end());
        for(char c : used)
            score += freq[c];
        return score;
    };

    sort(possible_words.begin(), possible_words.end(), [&](string a, string b){
        return score(a) > score(b);
    });

    for(auto& word : possible_words)
        cout << word << endl;

    return 0;
}
