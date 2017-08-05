# Handy for reformatting logging output to only focus on logs from the Z80.
# usage: awk -f reformatz80.awk log
# The output format is, each line:
# lineno: pc_address opcode mnemonic registers
# Here lineno refers to the line number in the original output file.

BEGIN {
  FS=": "
  looking_pc = 1
}

$2 == "Z80" && $3 == "PC" { 
    if (looking_pc) {
        looking_pc = 0
        printf("%10s: %2s", NR, $4)
    }
}

$2 == "Z80" && $3 == "opcode" {
    opcodes = opcodes " " $4
}

$2 == "Z80" && $3 == "op" {
    printf("%22-s :   %21-s\n", opcodes, $4)
    opcodes = ""
    looking_pc = 1
}

$2 == "Z80" && $3 == "status" {
    printf("%s  ", $4);
    # print "  " $4
}

#$1 == "Z80:" && /Reset/ {
#    $1 = "";
#    gsub(/^ */, "", $0);
#    printf("%7s: %s\n", NR, $0);
#}

#$1 == "Z80:" && /interrupt/ {
#    $1 = "";
#    gsub(/^ */, "", $0);
#    printf("%7s: %s\n", NR, $0);
#}

