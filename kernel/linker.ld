ENTRY(_start)
PHDRS { text PT_LOAD FLAGS(5); data PT_LOAD FLAGS(6); }
SECTIONS {
    . = 0x2000000;
    .text : ALIGN(4096) { *(.text .text.*) *(.rodata .rodata.*) } :text
    .data : ALIGN(4096) { *(.data .data.*) *(.got .got.*) } :data
    .bss : ALIGN(4096) { *(.bss .bss.*) *(COMMON) } :data
    /DISCARD/ : { *(.eh_frame*) *(.comment*) }
}
