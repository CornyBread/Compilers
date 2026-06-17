#Profe no nos acople
#Manejar de diferente manera el None

def factorial(n):
     resultado = 1
     while n > 1:
          resultado = resultado * n
          n -= 1
     return resultado


def main():
     numero = 5
     pi = 3.14
     mascara = 0xFF
     bits = 0b1010
     activo = True
     mensaje = "El factorial es:"
     if numero >= 0 and activo:
          print(mensaje, factorial(numero))
     else:
          print("Numero negativo")

$
"""


main()


