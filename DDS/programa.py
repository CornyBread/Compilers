
def factorial(n: int) -> int:
    resultado: int = 1
    while n > 1:
        resultado = resultado * n
        n -= 1
    return resultado


def main():
    numero: int = 5
    pi: float = 3.14
    formula = 1 / 3 + 4 * 3 + (8 / 4 * (6 - 2))
    activo: bool = True
    mensaje = "El factorial es:"
    if numero >= 0 and activo:
        print(mensaje, factorial(numero))
    elif numero == 0:
        print("cero")
    else:
        print("Numero negativo")
    for i in range(numero):
        print(i)


main()
