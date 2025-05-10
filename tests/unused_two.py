class Foo:
    def __init__(self):
        pass

    @staticmethod
    def save():
        pass


class Bar:
    def __init__(self):
        pass

    @staticmethod
    def save():
        pass


class Dar:
    def __init__(self):
        pass

    @staticmethod
    def save():
        pass


def save():
    pass


SAVE_FUNCTIONS = [save, Dar.save, Foo.save, Bar.save]
