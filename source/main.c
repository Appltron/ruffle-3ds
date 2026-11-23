#include <3ds.h>
#include <stdio.h>

int main(int argc, char **argv)
{
    (void)argc;
    (void)argv;

    gfxInitDefault();
    consoleInit(GFX_TOP, NULL);
    consoleInit(GFX_BOTTOM, NULL);

    printf("Ruffle3DS\n");
    printf("Native 3DS build smoke test\n\n");
    printf("O3DS is the primary target.\n");
    printf("No Flash runtime is integrated yet.\n\n");
    printf("Press START to exit.\n");

    while (aptMainLoop()) {
        hidScanInput();
        const u32 keys = hidKeysDown();

        if (keys & KEY_START) {
            break;
        }

        gfxFlushBuffers();
        gfxSwapBuffers();
        gspWaitForVBlank();
    }

    gfxExit();
    return 0;
}
