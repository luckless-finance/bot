#!/usr/bin/perl
########################################################
# extracts and prints command descriptions from Makefile
# assumes each recipe follows convention:
#
# <recipe-name>: <ignored>
# 	@echo "-------------------"
# 	@echo "<command description>"
# 	@echo "-------------------"
#   <ignored>
# 
########################################################

# https://www.perl.com/article/21/2013/4/21/Read-an-entire-file-into-a-string/
# print "read Makefile\n";
open my $fh, '<', 'Makefile' or die "Can't open file $!";
my $file_content = do { local $/; <$fh> };

# https://regexr.com/4usup
# $regex = '/^(.*):.*$\n\t@echo "-------------------"$\n\t@echo (.*)$\n\t@echo "-------------------"$\n\t/gm';
# print $regex;

# https://www.tutorialspoint.com/perl/perl_regular_expressions.htm
# $file_content =~ /^(.*):.*\n\t\@echo "-------------------"\n\t\@echo (.*)\n\t\@echo "-------------------"\n\t/gm;
# print "$1 - $2\n";


@matches = $file_content =~ /^(.*):.*\n\t\@echo "-------------------"\n\t\@echo (.*)\n\t\@echo "-------------------"\n\t/gm;
# https://www.tutorialspoint.com/perl/perl_arrays.htm
$command_count = @matches;
# https://www.perltutorial.org/perl-for-loop/
my @idxs = (0..$command_count-1);
for(my $idx = 0; $idx <= $#idxs; $idx++){
 print("$matches[$idx] - $matches[$idx + 1]\n");
 $idx++;
}