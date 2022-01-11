# Words

Words takes a word list, and produces a sequence of optimal guesses for word games similar to Wordle by Josh Wardle.

In this case, "optimal" means maximizing the information that the guess reveals about the puzzle. The guesses produced cover the entire alphabet, and focus on the most frequent letters first. 

You may also pass additional arguments, which are words from the word list you disallow. This is useful if your word list contains words that the puzzle's word list does not. 

Obviously, you won't win by just playing these guesses, as they depend only on the word list, not the puzzle. This is not a wordle solver.
