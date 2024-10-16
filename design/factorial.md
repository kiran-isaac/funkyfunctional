### 0: Input

factorial n :: Int -> Int
  if n <= 0 then -1
  else if n == 0 = 1
  else = n * factorial (n - 1) 

main args :: String -> Int
  factorial 5

---
### 1

factorial n :: Int -> Int
  where n <= 0 = -1
  where n == 0 = 1
  otherwise = n * factorial (n - 1) 

main args :: String -> Int
  where 5 <= 0 = -1
  where 5 == 0 = 1
  otherwise = 5 * factorial (5 - 1) 

---
### 2

factorial n :: Int -> Int
  where n <= 0 = -1
  where n == 0 = 1
  otherwise = n * factorial (n - 1) 

main args :: String -> Int
  5 * factorial (5 - 1) 

---
### 3

factorial n :: Int -> Int
  where n <= 0 = -1
  where n == 0 = 1
  otherwise = n * factorial (n - 1) 

main args :: String -> Int
  5 * factorial 4

---
### 4

factorial n :: Int -> Int
  where n <= 0 = -1
  where n == 0 = 1
  otherwise = n * factorial (n - 1) 

main args :: String -> Int
  5 * factorial 4