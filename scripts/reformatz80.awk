# Handy for reformatting logging output to only focus on logs from the Z80.
# usage: awk -f reformatz80.awk log
# The output format is, each line:
# lineno: pc_address opcode mnemonic registers
# Here lineno refers to the line number in the original output file.

BEGIN {
  FS=": "
}

$2 == "Z80" && $3 == "PC" { 
    printf("%10s:", NR, $4)
}

$2 == "Z80" && $3 == "opcode" {
    printf("  %12-s", $4)
}

$2 == "Z80" && $3 == "op" {
    printf("   %21-s", $4)
}

$2 == "Z80" && $3 == "status" {
    print "  " $4
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

