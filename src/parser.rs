/*
 * Parser Output:
 * Semantic actions should build an AST
 * The root of the AST (and only the root) should be of type program
 * For programs that have errors, the output is the error message
 * For multi line constructs free to use any line that is part of the construct
 * Parser needs to work for programs contained in a single file
 *
 * Error Handling:
 * The parser should recover at least in the following situations:
 * - if error in calss definition but class terminated properly and next class is syntactically correct,
 * the parser should be able to start from next class definition
 *
 * - similarly, parser should recover from errors in features (going on to the next feature),
 * a let biding (going on to the next variable), and an expression inside a { ... } block
 *
 *
 * Remarks:
 * - Only use precedence declarations for expressions
 * - the let construct introduces ambiguity, the manual says let expression extends as far to the right as possible
 * - Depending on the way your grammar is written, this ambiguity may show up
 * in your parser as a shift-reduce conflict involving the productions for let. If you run into such a conflict,
 * you might want to consider solving the problem by using a bison/CUP feature that allows precedence to
 * be associated with productions (not just operators) ??
 */
