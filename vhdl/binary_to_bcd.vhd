library IEEE;
use IEEE.STD_LOGIC_1164.ALL;
use IEEE.NUMERIC_STD.ALL;

entity binary_to_bcd is
    Port (
        clk           : in  std_logic;
        reset         : in  std_logic; 
        start         : in  std_logic; 
        input_number  : in  std_logic_vector(15 downto 0);
        output_number : out std_logic_vector(15 downto 0); 
        ready         : out std_logic  
    );
end binary_to_bcd;

architecture Behavioral of binary_to_bcd is
    type state_type is (IDLE, SHIFTING, DONE);
    signal state : state_type := IDLE;
    
    signal bcd_reg : unsigned(15 downto 0) := (others => '0');
    signal bin_reg : std_logic_vector(15 downto 0) := (others => '0');
    signal loop_count : integer range 0 to 16 := 0;

begin

    process (clk, reset)
  
        variable v_bcd : unsigned(15 downto 0);
    begin
        if reset = '1' then
            state <= IDLE;
            bcd_reg <= (others => '0');
            bin_reg <= (others => '0');
            ready <= '0';
        elsif rising_edge(clk) then
            case state is
                when IDLE =>
                    ready <= '1';
                    if start = '1' then
                        bin_reg <= input_number;
                        bcd_reg <= (others => '0');
                        loop_count <= 0;
                        state <= SHIFTING;
                        ready <= '0';
                    end if;

                when SHIFTING =>
                    v_bcd := bcd_reg;
                    if v_bcd(3 downto 0) > 4 then
                        v_bcd(3 downto 0) := v_bcd(3 downto 0) + 3;
                    end if;
                    if v_bcd(7 downto 4) > 4 then
                        v_bcd(7 downto 4) := v_bcd(7 downto 4) + 3;
                    end if;
                    if v_bcd(11 downto 8) > 4 then
                        v_bcd(11 downto 8) := v_bcd(11 downto 8) + 3;
                    end if;
                    if v_bcd(15 downto 12) > 4 then
                        v_bcd(15 downto 12) := v_bcd(15 downto 12) + 3;
                    end if;

                    v_bcd := shift_left(v_bcd, 1);                    
                    v_bcd(0) := bin_reg(15);
                    bin_reg <= bin_reg(14 downto 0) & '0';
                    bcd_reg <= v_bcd;                   
                    loop_count <= loop_count + 1;                    
                    if loop_count = 15 then
                        state <= DONE;
                    end if;
                when DONE =>
                    output_number <= std_logic_vector(bcd_reg);
                    ready <= '1';
                    state <= IDLE;
            end case;
        end if;
    end process;
end Behavioral;
