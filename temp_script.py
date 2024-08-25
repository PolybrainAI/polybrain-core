import onpy
partstudio = onpy.get_document('9cb45d508313846534155943').get_partstudio()
partstudio.wipe()
import onpy
partstudio = onpy.get_document("9cb45d508313846534155943").get_partstudio()

# Step 1: Create the Base Structure
heart_sketch = partstudio.add_sketch(plane=partstudio.features.front_plane, name="Heart Sketch")
line_a = heart_sketch.add_line((0, 0), (0, 5))
line_b = heart_sketch.add_line((0, 5), (1.5, 7.5))  # Create angled line up to the top left lobe
line_c = heart_sketch.add_line((1.5, 7.5), (3, 5))
line_d = heart_sketch.add_line((3, 5), (3, 0))
line_e = heart_sketch.add_line((3, 0), (0, 0))

# Step 2: Create Heart Lobes
circle_1 = heart_sketch.add_centerpoint_arc(centerpoint=(0.75, 7.5), radius=1.5, start_angle=0, end_angle=180)
circle_2 = heart_sketch.add_centerpoint_arc(centerpoint=(2.25, 7.5), radius=1.5, start_angle=0, end_angle=180)

# Step 3: Add Fillets
heart_sketch.add_fillet(line_b, line_e, radius=1.5)
heart_sketch.add_fillet(line_c, line_d, radius=1.5)

# Step 4: Copy and Mirror
heart_lobes = [circle_1, circle_2, line_b, line_c, line_d, line_e]
mirrored_items = heart_sketch.mirror(items=heart_lobes, line_point=(0, 5), line_dir=(1, 0), copy=True)

# Step 5: Extrude the Heart Shape
heart_extrude = partstudio.add_extrude(heart_sketch.faces.largest(), distance=1, name="Heart Extrude")