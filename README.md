
For every Asset $a$, compute `score`
```py
p = price(close)
ma50 = sma(p, 50)
sma200 = sma(p, 200)
score = ma50/(ma50 - sma200)
```
where `score = `$s_{a,t} \in \R$

```
filter(s, |x|x>0)


```