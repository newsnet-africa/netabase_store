from function_bench.confidence_interval import ConfidenceInterval
from function_bench.backend import Backend
from readline import backend
from function_bench.function_types import FunctionType


class FunctionResult:
    function_type: FunctionType
    backend: Backend
    result: ConfidenceInterval
