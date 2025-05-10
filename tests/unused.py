import json

from tests.unused_two import SAVE_FUNCTIONS


def first():
    print("first")


def second():
    print("second")


class Boom:
    def __init__(self):
        print("Boom")

    def bim(self):
        print("bim")


first()
boom = Boom()
boom.bim()
