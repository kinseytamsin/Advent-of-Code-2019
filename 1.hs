{-# LANGUAGE Safe #-}

import System.IO
import Data.Monoid

foldSum :: (Foldable t, Num c) => t c -> c
foldSum = getSum . foldMap Sum

fuelRequired :: Int -> Int
fuelRequired = subtract 2 . flip div 3

fuelRequiredRecursive :: Int -> Int
fuelRequiredRecursive = foldSum . takeWhile (>0) . fuelReqs
    where fuelReqs x = fuelRequired x : (fuelReqs . fuelRequired) x

main = do
    withFile "1.txt" ReadMode (\hdl -> do
        masses <- fmap (map (read :: String -> Int) . lines) $ hGetContents hdl
        putStrLnShow $ foldSumMap fuelRequired masses
        putStrLnShow $ foldSumMap fuelRequiredRecursive masses)
    where putStrLnShow = putStrLn . show
          foldSumMap f = foldSum . map f
