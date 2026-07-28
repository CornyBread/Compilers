public class Programa {

    public static int factorial(int n) {
        int resultado = 1;
        while ((n > 1)) {
            resultado = (resultado * n);
            n -= 1;
        }
        return resultado;
    }

    public static void main(String[] args) {
        int numero = 5;
        double pi = 3.14;
        var formula = ((((double)(1) / (3)) + (4 * 3)) + (((double)(8) / (4)) * (6 - 2)));
        boolean activo = true;
        var mensaje = "El factorial es:";
        if (((numero >= 0) && activo)) {
            System.out.println(mensaje + " " + factorial(numero));
        } else if ((numero == 0)) {
            System.out.println("cero");
        } else {
            System.out.println("Numero negativo");
        }
        for (int i = 0; i < numero; i++) {
            System.out.println(i);
        }
    }
}
