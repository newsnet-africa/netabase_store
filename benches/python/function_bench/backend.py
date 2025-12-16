from enum import Enum


class Backend:
    pass


class MainBackend(Enum, Backend):
    Redb = 0


class RawBackend(Enum, Backend):
    Redb = 0
