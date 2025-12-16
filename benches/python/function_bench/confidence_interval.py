from enum import Enum


class IntervalType(Enum):
    Typical = 1
    Mean = 2
    Median = 3
    MeanAbsDev = 4
    Slope = 5


class ConfidenceInterval:
    interval_type: IntervalType
    esitmate: float
    lower_bound: float
    upper_bound: float
