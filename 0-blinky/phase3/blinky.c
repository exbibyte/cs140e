#define GPIO_BASE (0x3F000000 + 0x200000)
#define true 1
#define false 0

volatile unsigned *GPIO_FSEL1 = (volatile unsigned *)(GPIO_BASE + 0x04);
volatile unsigned *GPIO_SET0  = (volatile unsigned *)(GPIO_BASE + 0x1C);
volatile unsigned *GPIO_CLR0  = (volatile unsigned *)(GPIO_BASE + 0x28);

static void spin_sleep_us(unsigned int us) {
    for (unsigned int i = 0; i < us * 6; i++) {
	asm volatile("nop");
    }
}

static void spin_sleep_ms(unsigned int ms) {
    spin_sleep_us(ms * 1000);
}

int main(void) {
    // STEP 1: Set GPIO Pin 16 as output.
    *GPIO_FSEL1 |= 0x01 << 18;
    
    // STEP 2: Continuously set and clear GPIO 16.
    while(true){
	*GPIO_SET0 |= 0x0f;
	spin_sleep_ms(250);
	*GPIO_CLR0 |= 0x0f;
	spin_sleep_ms(250);
    }
}
