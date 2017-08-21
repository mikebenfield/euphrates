# Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
# This file is part of Attalus. You may distribute and/or modify Attalus under
# the terms of the GNU General Public License as published by the Free Sofware
# Foundation, either version 3 of the license or (at your option) any later
# version. You should have received a copy of the GNU General Public License
# along with Attalus. If not, see <http://www.gnu.org/licenses/>.

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

# {
#     if doing_state >= 1 {
#         doing_state -= 1
#         printf("\n%s", $0)
#     }
# }

$2 == "Z80" && $3 == "state" {
    doing_state = 2
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

