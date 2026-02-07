library IEEE;
use IEEE.STD_LOGIC_1164.ALL;
use work.network_types.all;
use IEEE.numeric_std.all;

entity sorter is
    Port (
        CLK100MHZ: in std_logic;
        btnR: in std_logic;
        LED: out std_logic_vector(15 downto 0);
        seg: out std_logic_vector(6 downto 0);
        an: out std_logic_vector(3 downto 0)
     );
end sorter;

architecture Behavioral of sorter is

    constant N_SYSTEM : integer := 8;
    constant W_SYSTEM : integer := 16;

    signal inputs : mem(0 to N_SYSTEM - 1)(W_SYSTEM - 1 downto 0) := (
        std_logic_vector(to_unsigned(3246, 16)),
        std_logic_vector(to_unsigned(5749, 16)),
        std_logic_vector(to_unsigned(1368, 16)),
        std_logic_vector(to_unsigned(7925, 16)),
        std_logic_vector(to_unsigned(411, 16)),
        std_logic_vector(to_unsigned(3529, 16)),
        std_logic_vector(to_unsigned(8559, 16)),
        std_logic_vector(to_unsigned(2190, 16))
    );

    signal outputs : mem(0 to N_SYSTEM - 1)(W_SYSTEM - 1 downto 0) := (others => (others => '0'));
    signal enabled: std_logic := '0';
    signal idx: integer range 0 to N_SYSTEM - 1 := 0;
    signal led_reg: std_logic_vector(15 downto 0) := (others => '0');
    signal reset: std_logic := '0';
    signal start: std_logic := '0';
    signal output_number: std_logic_vector(15 downto 0) := (others => '0');
    signal ready: std_logic := '0';

begin
    network: entity work.bitonic_network
    generic map (
        n => N_SYSTEM,
        width => W_SYSTEM
    )
    port map(
        clk => CLK100MHZ,
        inputs => inputs,
        outputs => outputs
    );

    push_btn: entity work.push_btn port map(clk => CLK100MHZ, btn => btnR, enabled => enabled);
    binary_to_bcd: entity work.binary_to_bcd port map(clk => CLK100MHZ, 
    reset => reset, 
    start => start, 
    input_number => led_reg, 
    output_number => output_number, 
    ready => ready);
    display: entity work.seven_segment_display port map(clk => CLK100MHZ, 
    number => output_number, seg => seg, an => an );
    process (CLK100MHZ)
    begin
        if rising_edge(CLK100MHZ) then
            if enabled = '1' then
                start <= '1';
                led_reg <= std_logic_vector(resize(unsigned(outputs(idx)), 16));

                if idx = N_SYSTEM - 1 then
                    idx <= 0;
                else
                    idx <= idx + 1;
                end if;
            
            else
                start <= '0';
            end if; 
        end if;
    end process;

    LED <= led_reg;
end Behavioral;
