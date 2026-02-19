library IEEE;
use IEEE.STD_LOGIC_1164.ALL;
use IEEE.NUMERIC_STD.ALL;

entity push_btn is
    Port ( 
        clk     : in  std_logic; 
        btn     : in  std_logic; 
        enabled : out std_logic
    );
end push_btn;

architecture Behavioral of push_btn is
    signal btn_sync_0, btn_sync_1 : std_logic := '0';
    signal count : unsigned(19 downto 0) := (others => '0');
    signal btn_stable : std_logic := '0';
    signal btn_last   : std_logic := '0';
begin

    process (clk)
    begin
        if rising_edge(clk) then

            btn_sync_0 <= btn;
            btn_sync_1 <= btn_sync_0;

            if btn_sync_1 /= btn_stable then
                count <= count + 1;
                if count = 1000000 then 
                    btn_stable <= btn_sync_1;
                    count <= (others => '0');
                end if;
            else
                count <= (others => '0');
            end if;

            btn_last <= btn_stable;
            enabled <= '0';
            if btn_stable = '1' and btn_last = '0' then
                enabled <= '1';
            end if;
        end if;
    end process;

end Behavioral;
