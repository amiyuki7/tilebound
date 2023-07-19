import heapq
import pygame
from math import hypot

# Initialize Pygame
pygame.init()

# Define the screen size
screen_width = 800
screen_height = 600

# Create the screen
screen = pygame.display.set_mode((screen_width, screen_height))

# Set the caption of the screen
pygame.display.set_caption("My Pygame")

class HexCell:
    def __init__(self, q, r, cost, is_obstacle=False):
        self.q = q
        self.r = r
        self.display_coords = ((self.q + 2) * 50 + self.r * 25, (self.r + 2) * 50)
        self.cost = cost
        self.is_obstacle = is_obstacle
        self.g = 0
        self.h = 0
        self.f = 0
        self.parent = None
    def update(self):
        global goal_cell
        (x, y) = pygame.mouse.get_pos()
        print(self.q, self.r, path)
        if hypot(self.display_coords[0] - x, self.display_coords[1] - y) < 25 and not self.is_obstacle:
            goal_cell = self
        colour = (100, 100, 100)
        if self == start_cell:
            colour = (255, 255, 0)
        elif self == goal_cell:
            colour = (255, 0, 255)
        elif self.is_obstacle:
            colour = (150, 0, 0)
        elif (self.q, self.r) in path:
            colour = (200, 200, 200)
        pygame.draw.circle(screen, colour, self.display_coords, 25)
    

    def __lt__(self, other):
        return self.f < other.f

def calculate_hex_distance(cell1, cell2):
    # Calculate hexagonal distance between two cells
    dq = abs(cell1.q - cell2.q)
    dr = abs(cell1.r - cell2.r)
    return max(dq, dr, abs(cell1.q + cell1.r - cell2.q - cell2.r))

def get_hex_neighbors(cell):
    # Generate neighbors for a hexagonal cell
    neighbors = []
    q, r = cell.q, cell.r
    offsets = [(0, -1), (1, -1), (1, 0), (0, 1), (-1, 1), (-1, 0)]
    for dq, dr in offsets:
        neighbor_q, neighbor_r = q + dq, r + dr
        neighbors.append((neighbor_q, neighbor_r))
    return neighbors

def a_star_hexagonal(start, goal):
    open_list = []
    closed_set = set()

    heapq.heappush(open_list, start)
    while open_list:
        current_cell = heapq.heappop(open_list)
        closed_set.add(current_cell)

        if current_cell == goal:
            # Path found, reconstruct it
            path = []
            while current_cell:
                path.append((current_cell.q, current_cell.r))
                current_cell = current_cell.parent
            return path[::-1]

        neighbors = get_hex_neighbors(current_cell)
        for neighbor_q, neighbor_r in neighbors:
            neighbor = hexagonal_grid.get((neighbor_q, neighbor_r))
            if not neighbor or neighbor.is_obstacle or neighbor in closed_set:
                continue

            g = current_cell.g + neighbor.cost
            h = calculate_hex_distance(neighbor, goal)
            f = g + h

            if neighbor in open_list and f >= neighbor.f:
                continue

            neighbor.g = g
            neighbor.h = h
            neighbor.f = f
            neighbor.parent = current_cell

            if neighbor not in open_list:
                heapq.heappush(open_list, neighbor)

    return None

hexagonal_grid = {}
# Example usage
for y in range(7):
    for x in range(8):
        hexagonal_grid[(x, y)] = HexCell(x, y, 1)


hexagonal_grid[(3,2)].is_obstacle = True
hexagonal_grid[(2,2)].is_obstacle = True
hexagonal_grid[(3,0)].is_obstacle = True
hexagonal_grid[(4,0)].is_obstacle = True
hexagonal_grid[(4,1)].is_obstacle = True

start_cell = hexagonal_grid[(0, 0)]
goal_cell = hexagonal_grid[(5, 6)]

path = []

running = True
while running:
    path = a_star_hexagonal(start_cell, goal_cell)
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            running = False
            
    screen.fill((255, 255, 255))
    
    for tile in hexagonal_grid:
        hexagonal_grid[tile].update()
    
    pygame.display.flip()