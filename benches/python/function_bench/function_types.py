from abc import ABC
from enum import Enum


class FunctionType:
    pass


class CRUDFunction(Enum, FunctionType):
    Create = 0
    Read = 1
    Update = 2
    Delete = 3
