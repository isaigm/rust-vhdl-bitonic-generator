library IEEE;
use IEEE.STD_LOGIC_1164.ALL;
use work.network_types.all;
use IEEE.numeric_std.all;

entity sorter is
    Port (
        CLK100MHZ: in std_logic;
        btnR: in std_logic;
        LED: out std_logic_vector(15 downto 0)
     );
end sorter;

architecture Behavioral of sorter is
    
    constant N_SYSTEM : integer := 8;
    constant W_SYSTEM : integer := 16;

    signal inputs : mem(0 to N_SYSTEM - 1)(W_SYSTEM - 1 downto 0) := (
        std_logic_vector(to_unsigned(24470, 16)),
        std_logic_vector(to_unsigned(32931, 16)),
        std_logic_vector(to_unsigned(29393, 16)),
        std_logic_vector(to_unsigned(3328, 16)),
        std_logic_vector(to_unsigned(7941, 16)),
        std_logic_vector(to_unsigned(64973, 16)),
        std_logic_vector(to_unsigned(47855, 16)),
        std_logic_vector(to_unsigned(41532, 16))
    );
    
    signal outputs : mem(0 to N_SYSTEM - 1)(W_SYSTEM - 1 downto 0) := (others => (others => '0'));
    signal enabled: std_logic := '0';
    signal idx: integer range 0 to N_SYSTEM - 1 := 0;
    signal led_reg: std_logic_vector(15 downto 0) := (others => '0');

begin
    network: entity work.bitonic_network 
    generic map (
        n => N_SYSTEM,
        width => W_SYSTEM
    )
    port map(
        inputs => inputs, 
        outputs => outputs
    );

    push_btn: entity work.push_btn port map(CLk100MHZ => CLK100MHZ, btn => btnR, enabled => enabled);

    process (CLK100MHZ)
    begin
        if rising_edge(CLK100MHZ) then
            if enabled = '1' then
              
                led_reg <= std_logic_vector(resize(unsigned(outputs(idx)), 16));
                
                if idx = N_SYSTEM - 1 then
                    idx <= 0;    
                else
                    idx <= idx + 1;
                end if;
            end if;
        end if;
    end process;
    
    LED <= led_reg;
end Behavioral;
