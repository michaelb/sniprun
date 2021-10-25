import numpy as np


def fact(n):
    if type(n) is not int or n < 0:
        print("Argument must be a positive integer")
    if n == 0:
        return 1
    return n * fact(n-1)


# Can you spot the error in fact()?

There can be invalid & unfinished code here
a = 5 / 0


# Now testing things





