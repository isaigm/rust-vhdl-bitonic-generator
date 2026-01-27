library IEEE;
use IEEE.STD_LOGIC_1164.ALL;
use IEEE.NUMERIC_STD.ALL;


entity comparator is
    Port ( 
        in_A  : in  std_logic_vector(15 downto 0);
        in_B  : in  std_logic_vector(15 downto 0);
        dir   : in  std_logic; 
        out_L : out std_logic_vector(15 downto 0); 
        out_H : out std_logic_vector(15 downto 0)
    );
end comparator;

architecture Behavioral of comparator is
begin
    process(in_A, in_B, dir)
    begin
    
        out_L <= in_A;
        out_H <= in_B;

        if (dir = '1' and unsigned(in_A) > unsigned(in_B)) then 
            out_L <= in_B;
            out_H <= in_A;
            
        elsif (dir = '0' and unsigned(in_A) < unsigned(in_B)) then 
            out_L <= in_B;
            out_H <= in_A;
        end if;
        
    end process;
end Behavioral;