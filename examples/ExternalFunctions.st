@EXTERNAL FUNCTION MYLOG : DINT
VAR_INPUT
    x : DINT;
END_VAR
END_FUNCTION

@EXTERNAL FUNCTION MYPRINTF : DINT
VAR_INPUT
    text : STRING;
    value : DINT;
END_VAR
END_FUNCTION


FUNCTION main : DINT
	main := MYLOG(100);
  MYPRINTF('Log value %d\n', main);
END_FUNCTION
