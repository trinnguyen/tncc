int g = 1;
int g2 = 101;
int foo(int a)
{
    int g = 2;
    int t = 11;
    {
        int g = 3;
        int c = a;
    }

    return t;
}

int main()
{
    return foo(51);
}