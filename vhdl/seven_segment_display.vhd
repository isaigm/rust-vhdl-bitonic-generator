library IEEE;
use IEEE.STD_LOGIC_1164.ALL;
use IEEE.NUMERIC_STD.ALL;
entity seven_segment_display is
    Port (
        clk: in std_logic;
        number: in std_logic_vector(15 downto 0);
        seg: out std_logic_vector(6 downto 0);
        an: out std_logic_vector(3 downto 0)
    );
end seven_segment_display;

architecture Behavioral of seven_segment_display is
    signal s_segments: std_logic_vector(6 downto 0) := (others => '1');
    signal s_an: std_logic_vector(3 downto 0) := (others => '1');
    signal s_counter : unsigned(31 downto 0) := (others => '0'); 
    constant cycles : integer := 100_000;
    type state_type is (COUNTING);
    type digit_type is (DIGIT1, DIGIT2, DIGIT3, DIGIT4);
    signal digit: digit_type := DIGIT1;
    signal state : state_type := COUNTING;
    signal enable_encoder: std_logic := '0';
    signal s_digit: std_logic_vector(3 downto 0) := (others => '0');
    
begin
    ss_encoder: entity work.seven_segment_encoder port map(enable => enable_encoder, digit => s_digit, segments => s_segments);
    process(clk)
    begin
    if rising_edge(clk) then
        case state is 
        when COUNTING =>
            if s_counter >= cycles then
                s_counter <= (others => '0');
                enable_encoder <= '1';
                case digit is
                when DIGIT1 =>
                    s_digit <= number(3 downto 0);
                    s_an <= "1110";
                    digit <= DIGIT2;
                when DIGIT2 =>
                    s_digit <= number(7 downto 4);
                    s_an <= "1101";
                    digit <= DIGIT3;
                when DIGIT3 =>
                    s_digit <= number(11 downto 8);
                    s_an <= "1011";
                    digit <= DIGIT4;
                when DIGIT4 =>
                    s_digit <= number(15 downto 12);
                    s_an <= "0111";
                    digit <= DIGIT1;
                end case;
            else 
                s_counter <= s_counter + 1;        
            end if;
        end case;
    end if;
    end process;
    seg <= s_segments;
    an <= s_an;
end Behavioral;
